# frozen_string_literal: true

# Reject movable third-party GitHub Action refs in workflows and composite actions.
# Psych is part of Ruby, so the CI gate needs no downloaded YAML parser.
require 'psych'

module ActionPins
  ACTION_REF = %r{\A[^/\s@]+/[^@\s]+@[0-9a-f]{40}\z}

  module_function

  def mapping_values(node, name)
    return [] unless node.is_a?(Psych::Nodes::Mapping)

    node.children.each_slice(2).filter_map do |key, value|
      value if key.is_a?(Psych::Nodes::Scalar) && key.value == name
    end
  end

  def composite_action?(runs)
    mapping_values(runs, 'using').any? do |using|
      using.is_a?(Psych::Nodes::Scalar) && using.value == 'composite'
    end
  end

  def check_invocation(node, path, errors)
    if node.is_a?(Psych::Nodes::Alias)
      errors << "#{path}:#{node.start_line + 1}: YAML aliases cannot stand in for action invocations"
      return
    end
    return unless node.is_a?(Psych::Nodes::Mapping)

    node.children.each_slice(2) do |key, value|
      next unless key.is_a?(Psych::Nodes::Scalar) && key.value == 'uses'

      ref = value.is_a?(Psych::Nodes::Scalar) ? value.value : nil
      unless ref&.start_with?('./', 'actions/') || ACTION_REF.match?(ref.to_s)
        errors << "#{path}:#{key.start_line + 1}: third-party action must use a full 40-character commit SHA: #{ref.inspect}"
      end
    end
  end

  def check_steps(node, path, errors)
    if node.is_a?(Psych::Nodes::Alias)
      errors << "#{path}:#{node.start_line + 1}: YAML aliases cannot stand in for action steps"
    elsif node.is_a?(Psych::Nodes::Sequence)
      node.children.each { |step| check_invocation(step, path, errors) }
    end
  end

  def check_workflow(root, path, errors)
    mapping_values(root, 'jobs').each do |jobs|
      if jobs.is_a?(Psych::Nodes::Alias)
        errors << "#{path}:#{jobs.start_line + 1}: YAML aliases cannot stand in for workflow jobs"
        next
      end
      next unless jobs.is_a?(Psych::Nodes::Mapping)

      jobs.children.each_slice(2) do |_name, job|
        check_invocation(job, path, errors)
        mapping_values(job, 'steps').each { |steps| check_steps(steps, path, errors) }
      end
    end
  end

  def check_composite(root, path, errors)
    mapping_values(root, 'runs').each do |runs|
      if runs.is_a?(Psych::Nodes::Alias)
        errors << "#{path}:#{runs.start_line + 1}: YAML aliases cannot stand in for action runs"
      elsif composite_action?(runs)
        mapping_values(runs, 'steps').each { |steps| check_steps(steps, path, errors) }
      end
    end
  end

  def check_file(root, relative_path, composite_only:)
    document = Psych.parse(File.read(File.join(root, relative_path)), filename: relative_path)
    return ["#{relative_path}:1: empty YAML document"] unless document

    content = document.children.first
    errors = []
    if composite_only
      check_composite(content, relative_path, errors)
    else
      check_workflow(content, relative_path, errors)
    end
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
