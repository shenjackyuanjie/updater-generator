use gix_hash::{oid, ObjectId};
use gix_object::{bstr::ByteSlice, tree::EntryMode, TreeRefIter};
use gix_odb::pack::Find;
use std::collections::HashMap;

mod changes;

// use pyo3::prelude::*;
//

fn hex_to_id(hex: &str) -> ObjectId {
    ObjectId::from_hex(hex.as_bytes()).expect("40 bytes hex")
}

fn head_of(db: &gix_odb::Handle) -> ObjectId {
    ObjectId::from_hex(
        std::fs::read(
            db.store_ref()
                .path()
                .parent()
                .unwrap()
                .join("refs")
                .join("heads")
                .join("main"),
        )
        .expect("head ref")
        .as_bstr()
        .trim(),
    )
    .expect("valid hex id")
}

fn all_commits(db: &gix_odb::Handle) -> HashMap<String, ObjectId> {
    use gix_traverse::commit;
    let mut buf = Vec::new();

    let head = head_of(db);
    commit::Ancestors::new(
        Some(head),
        commit::ancestors::State::default(),
        |oid, buf| {
            use gix_odb::FindExt;
            db.find_commit_iter(oid, buf)
        },
    )
    .collect::<Result<Vec<_>, _>>()
    .expect("valid iteration")
    .into_iter()
    .map(|c| {
        use gix_odb::FindExt;
        (
            db.find_commit(&c.id, &mut buf)
                .unwrap()
                .message
                .trim()
                .to_str_lossy()
                .into_owned(),
            c.id,
        )
    })
    .rev()
    .collect()
}

#[test]
fn diff() {
    use gix_diff::tree::{Changes, Recorder};
    use gix_odb::pack::FindExt;
    // 从仓库中读取两个树对象
    let mut buffer = Vec::new();
    let mut repo = gix_odb::at("../../tests/try_diff/.git/objects").unwrap();
    let tree_id = repo
        .try_find(
            &hex_to_id("9d394f2fd6abb585e4126f9995ca2e187d164900"),
            &mut buffer,
        )
        .unwrap()
        .unwrap()
        .0
        .decode()
        .unwrap()
        .into_commit()
        .unwrap()
        .tree();
    let tree = repo.find_tree(&tree_id, &mut buffer).unwrap().0;

    // let mut record = Recorder::track_location();
}
