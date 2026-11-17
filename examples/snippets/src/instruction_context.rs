use shipstern_core::{instruction::InstructionUpdate, ParseError, TransactionUpdate};

// #region instruction-logs
pub fn log_instruction(raw: &InstructionUpdate) {
    tracing::info!(path = ?raw.path, logs = ?raw.log_messages());

    let direct_logs: Vec<_> = raw.direct_log_messages().collect();
    tracing::info!(path = ?raw.path, ?direct_logs);
}
// #endregion instruction-logs

// #region instruction-tree
pub fn inspect_transaction(raw: &TransactionUpdate) -> Result<(), ParseError> {
    let outer = InstructionUpdate::build_from_txn(raw)?;

    for root in &outer {
        for (instruction, flat_index) in root.visit_all_with_flat_indices() {
            tracing::info!(
                tree_path = ?instruction.path,
                ?flat_index,
                program = %instruction.program,
            );
        }
    }
    Ok(())
}
// #endregion instruction-tree

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use shipstern_core::{
        instruction::{InstructionShared, Path},
        Pubkey,
    };

    use super::*;

    fn instruction(shared: &Arc<InstructionShared>, path: Vec<u32>) -> InstructionUpdate {
        InstructionUpdate {
            program: Pubkey::new([0; 32]),
            accounts: vec![],
            data: vec![],
            shared: Arc::clone(shared),
            inner: vec![],
            path: Path::from(path),
            log_range: 0..0,
        }
    }

    #[test]
    fn nested_context_keeps_log_scope_and_distinguishes_flat_positions() {
        let logs = [
            "Program 11111111111111111111111111111111 invoke [1]",
            "Program log: parent before",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
            "Program log: child",
            "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
            "Program log: parent after",
            "Program 11111111111111111111111111111111 success",
            "Program log: another instruction",
        ];
        let shared = Arc::new(InstructionShared {
            log_messages: logs.iter().map(|line| (*line).to_owned()).collect(),
            ..Default::default()
        });
        let mut root = instruction(&shared, vec![2]);
        root.log_range = 0..7;
        let first = instruction(&shared, vec![2, 0]);
        let mut second = instruction(&shared, vec![2, 1]);
        second.inner.push(instruction(&shared, vec![2, 1, 0]));
        root.inner = vec![first, second];

        assert_eq!(root.log_messages().len(), 7);
        assert_eq!(
            root.direct_log_messages().collect::<Vec<_>>(),
            vec![logs[0], logs[1], logs[5], logs[6]]
        );
        let positions: Vec<_> = root
            .visit_all_with_flat_indices()
            .map(|(node, flat)| (format!("{:?}", node.path), format!("{flat:?}")))
            .collect();
        assert_eq!(
            positions,
            vec![
                ("3".into(), "3".into()),
                ("3.1".into(), "3.1".into()),
                ("3.2".into(), "3.2".into()),
                ("3.2.1".into(), "3.3".into()),
            ]
        );
    }
}
