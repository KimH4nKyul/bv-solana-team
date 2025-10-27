use header_fork_choice::{BlockHeader, HeaderForkChoice, ReorgOutcome};

fn make_header(hash: &str, parent: Option<&str>, number: u64, difficulty: u64) -> BlockHeader {
    BlockHeader {
        hash: hash.to_string(),
        parent_hash: parent.map(|p| p.to_string()),
        number,
        difficulty,
    }
}

#[test]
fn simple_extension_returns_extended() {
    let genesis = make_header("genesis", None, 0, 10);
    let genesis_hash = genesis.hash.clone();
    let mut fork_choice = HeaderForkChoice::new(genesis.clone());

    let child = make_header("child-1", Some(&genesis_hash), 1, 3);
    let child_hash = child.hash.clone();

    let outcome = fork_choice.try_insert(child.clone()).expect("insert child");
    match outcome {
        ReorgOutcome::Extended { new_head } => assert_eq!(new_head.hash, child_hash),
        other => panic!("expected Extended outcome, got {:?}", other),
    }

    let canonical: Vec<_> = fork_choice.canonical_hashes().cloned().collect();
    assert_eq!(canonical, vec![genesis_hash, child.hash.clone()]);
    assert_eq!(fork_choice.head().number, 1);
}

#[test]
fn weaker_fork_does_not_reorganize() {
    let genesis = make_header("genesis", None, 0, 10);
    let genesis_hash = genesis.hash.clone();
    let mut fork_choice = HeaderForkChoice::new(genesis.clone());

    let strong_child = make_header("strong-child", Some(&genesis_hash), 1, 4);
    let strong_child_hash = strong_child.hash.clone();
    assert!(matches!(
        fork_choice.try_insert(strong_child.clone()),
        Ok(ReorgOutcome::Extended { .. })
    ));

    let weak_child = make_header("weak-child", Some(&genesis_hash), 1, 1);
    let outcome = fork_choice
        .try_insert(weak_child)
        .expect("insert weak fork");
    assert!(matches!(outcome, ReorgOutcome::NoReorg));

    assert_eq!(fork_choice.canonical_hashes().count(), 2);
    assert_eq!(fork_choice.head().hash.clone(), strong_child_hash);
}

#[test]
fn heavier_side_chain_triggers_reorg() {
    let genesis = make_header("genesis", None, 0, 10);
    let genesis_hash = genesis.hash.clone();
    let mut fork_choice = HeaderForkChoice::new(genesis.clone());

    let branch_a1 = make_header("branch-a1", Some(&genesis_hash), 1, 2);
    let branch_a1_hash = branch_a1.hash.clone();
    fork_choice
        .try_insert(branch_a1.clone())
        .expect("insert branch a1");

    let branch_a2 = make_header("branch-a2", Some(&branch_a1_hash), 2, 2);
    let branch_a2_hash = branch_a2.hash.clone();
    fork_choice
        .try_insert(branch_a2.clone())
        .expect("insert branch a2");

    let heavy_branch = make_header("branch-b1", Some(&genesis_hash), 1, 6);
    let heavy_branch_hash = heavy_branch.hash.clone();

    let outcome = fork_choice
        .try_insert(heavy_branch.clone())
        .expect("insert heavier fork");
    match outcome {
        ReorgOutcome::Reorganized {
            new_head,
            old_head,
            depth,
        } => {
            assert_eq!(new_head.hash, heavy_branch_hash);
            assert_eq!(old_head.hash, branch_a2_hash);
            assert_eq!(depth, 2);
        }
        other => panic!("expected Reorganized outcome, got {:?}", other),
    }

    let canonical: Vec<_> = fork_choice.canonical_hashes().cloned().collect();
    assert_eq!(canonical, vec![genesis_hash, heavy_branch.hash.clone()]);
    assert_eq!(fork_choice.head().number, 1);
}
