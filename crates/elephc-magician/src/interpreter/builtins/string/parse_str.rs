//! Purpose:
//! Parses URL-encoded query strings into a PHP array for eval `parse_str()` calls.
//!
//! Called from:
//! - The eval expression dispatcher for the mandatory by-reference output form.

use super::super::super::*;

/// Evaluates PHP 8's mandatory-output `parse_str(string $string, array &$result)` form.
pub(in crate::interpreter) fn eval_builtin_parse_str_call(
    args: &[EvalCallArg],
    context: &mut ElephcEvalContext,
    scope: &mut ElephcEvalScope,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let evaluated = eval_call_arg_values(args, context, scope, values)?;
    let (bound, _) = bind_evaluated_ref_builtin_args(&["string", "result"], &evaluated, false)?;
    let source = required_evaluated_ref_arg(&bound, 0)?.value;
    let output = required_evaluated_ref_arg(&bound, 1)?;
    let target = output.ref_target.clone().ok_or(EvalStatus::RuntimeFatal)?;
    let result = eval_parse_str_result(source, values)?;
    eval_write_direct_ref_target(
        &target,
        result,
        context,
        values,
        Some(ScopeCellOwnership::Owned),
    )?;
    values.null()
}

/// Parses the default PHP query-string grammar into a fresh associative result array.
fn eval_parse_str_result(
    source: RuntimeCellHandle,
    values: &mut impl RuntimeValueOps,
) -> Result<RuntimeCellHandle, EvalStatus> {
    let source = values.string_bytes(source)?;
    let mut root = QueryNode::Map(Vec::new());
    for field in source.split(|byte| matches!(byte, b'&' | b';')) {
        if field.is_empty() {
            continue;
        }
        let (raw_name, raw_value) = field
            .iter()
            .position(|byte| *byte == b'=')
            .map(|index| (&field[..index], &field[index + 1..]))
            .unwrap_or((field, &[]));
        let name = decode_form_component(raw_name);
        if name.is_empty() {
            continue;
        }
        let value = decode_form_component(raw_value);
        let Some(path) = query_key_path(&name) else {
            continue;
        };
        root.insert(&path, value);
    }
    root.materialize(values)
}

#[derive(Clone, PartialEq, Eq)]
enum QueryKey {
    Name(Vec<u8>),
    Append,
}

enum QueryNode {
    Scalar(Vec<u8>),
    Map(Vec<(Vec<u8>, QueryNode)>),
}

impl QueryNode {
    fn insert(&mut self, path: &[QueryKey], value: Vec<u8>) {
        let Some((head, tail)) = path.split_first() else {
            *self = Self::Scalar(value);
            return;
        };
        let Self::Map(entries) = self else {
            *self = Self::Map(Vec::new());
            self.insert(path, value);
            return;
        };
        let key = match head {
            QueryKey::Name(key) => key.clone(),
            QueryKey::Append => next_append_key(entries),
        };
        if let Some((_, child)) = entries.iter_mut().find(|(existing, _)| *existing == key) {
            child.insert(tail, value);
        } else {
            let mut child = Self::Map(Vec::new());
            child.insert(tail, value);
            entries.push((key, child));
        }
    }

    fn materialize(&self, values: &mut impl RuntimeValueOps) -> Result<RuntimeCellHandle, EvalStatus> {
        match self {
            Self::Scalar(value) => values.string_bytes_value(value),
            Self::Map(entries) => {
                let mut result = values.assoc_new(entries.len())?;
                for (key, value) in entries {
                    let key = values.string_bytes_value(key)?;
                    let value = value.materialize(values)?;
                    result = values.array_set(result, key, value)?;
                }
                Ok(result)
            }
        }
    }
}

fn next_append_key(entries: &[(Vec<u8>, QueryNode)]) -> Vec<u8> {
    let next = entries
        .iter()
        .filter_map(|(key, _)| std::str::from_utf8(key).ok()?.parse::<u64>().ok())
        .max()
        .map_or(0, |value| value.saturating_add(1));
    next.to_string().into_bytes()
}

fn query_key_path(name: &[u8]) -> Option<Vec<QueryKey>> {
    let root_end = name.iter().position(|byte| *byte == b'[').unwrap_or(name.len());
    let mut root = name[..root_end].to_vec();
    for byte in &mut root {
        if matches!(*byte, b' ' | b'.') { *byte = b'_'; }
    }
    if root.is_empty() { return None; }
    let mut path = vec![QueryKey::Name(root)];
    let mut rest = &name[root_end..];
    while let Some(after_open) = rest.strip_prefix(b"[") {
        let end = after_open.iter().position(|byte| *byte == b']')?;
        let key = &after_open[..end];
        path.push(if key.is_empty() { QueryKey::Append } else { QueryKey::Name(key.to_vec()) });
        rest = &after_open[end + 1..];
    }
    rest.is_empty().then_some(path)
}

fn decode_form_component(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'+' { output.push(b' '); index += 1; continue; }
        if input[index] == b'%' && index + 2 < input.len() {
            if let (Some(high), Some(low)) = (hex(input[index + 1]), hex(input[index + 2])) {
                output.push(high << 4 | low); index += 3; continue;
            }
        }
        output.push(input[index]); index += 1;
    }
    output
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
