# frozen_string_literal: true

# Reject movable third-party GitHub Action refs in workflows and composite actions.
# Psych is part of Ruby, so the CI gate needs no downloaded YAML parser.
require 'psych'

module ActionPins
  ACTION_REF = %r{\A[^/\s@]+/[^@\s]+@[0-9a-f]{40}\z}

  module_function

  def mapping_value(node, name)
    return nil unless node.is_a?(Psych::Nodes::Mapping)

    node.children.each_slice(2) do |key, value|
      return value if key.is_a?(Psych::Nodes::Scalar) && key.value == name
    end
    nil
  end

  def composite_action?(root)
    runs = mapping_value(root, 'runs')
    using = mapping_value(runs, 'using')
    using.is_a?(Psych::Nodes::Scalar) && using.value == 'composite'
  end

  def check_node(node, path, errors, check_uses:)
    case node
    when Psych::Nodes::Alias
      errors << "#{path}:#{node.start_line + 1}: YAML aliases cannot be checked for action pins"
    when Psych::Nodes::Mapping
      node.children.each_slice(2) do |key, value|
        if check_uses && key.is_a?(Psych::Nodes::Scalar) && key.value == 'uses'
          ref = value.is_a?(Psych::Nodes::Scalar) ? value.value : nil
          unless ref&.start_with?('./', 'actions/') || ACTION_REF.match?(ref.to_s)
            errors << "#{path}:#{key.start_line + 1}: third-party action must use a full 40-character commit SHA: #{ref.inspect}"
          end
        end
        check_node(value, path, errors, check_uses: check_uses)
      end
    when Psych::Nodes::Sequence
      node.children.each { |child| check_node(child, path, errors, check_uses: check_uses) }
    end
  end

  def check_file(root, relative_path, composite_only:)
    document = Psych.parse(File.read(File.join(root, relative_path)), filename: relative_path)
    return ["#{relative_path}:1: empty YAML document"] unless document

    content = document.children.first
    errors = []
    check_node(content, relative_path, errors, check_uses: !composite_only || composite_action?(content))
    errors
  rescue Psych::SyntaxError => error
    ["#{relative_path}:#{error.line}: invalid YAML: #{error.problem}"]
  end

  def check_repository(root)
    workflows = Dir.glob('.github/workflows/*.{yml,yaml}', base: root)
    actions = Dir.glob('**/action.{yml,yaml}', base: root, flags: File::FNM_DOTMATCH)
    workflows.sort.flat_map { |path| check_file(root, path, composite_only: false) } +
      actions.sort.flat_map { |path| check_file(root, path, composite_only: true) }
  end
end

if $PROGRAM_NAME == __FILE__
  root = ARGV.fetch(0, File.expand_path('../..', __dir__))
  errors = ActionPins.check_repository(root)
  if errors.empty?
    puts 'GitHub Action refs are pinned'
  else
    warn errors.join("\n")
    exit 1
  end
end
