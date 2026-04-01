// (#!#tep:entity.links)
// [#!#tep:entity.links](entity.links)
use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Context, Result};

use crate::entity::{Entity, EntityLink, EntityLookup};
use crate::repository::entity_repository::EntityRepository;
use crate::service::entity_service::LinkedEntityContext;

pub fn collect_link_context(
    repo: &EntityRepository<'_>,
    root_entity_id: i64,
    link_depth: usize,
) -> Result<Vec<LinkedEntityContext>> {
    if link_depth == 0 {
        return Ok(Vec::new());
    }

    let root_entity = repo
        .find(&EntityLookup::Id(root_entity_id))?
        .context("root entity not found")?;

    let mut discovered: HashMap<i64, LinkedEntityContext> = HashMap::new();
    let mut queued = HashSet::from([root_entity_id]);
    let mut queue = VecDeque::from([(root_entity_id, 0usize)]);

    while let Some((entity_id, depth)) = queue.pop_front() {
        if depth >= link_depth {
            continue;
        }

        enqueue_neighbors(repo, entity_id, depth + 1, &root_entity, &mut discovered, &mut queued, &mut queue)?;
    }

    let mut linked_entities = discovered.into_values().collect::<Vec<_>>();
    linked_entities.sort_by_key(|item| (item.depth, item.entity.entity_id));
    Ok(linked_entities)
}

fn enqueue_neighbors(
    repo: &EntityRepository<'_>,
    entity_id: i64,
    depth: usize,
    root_entity: &Entity,
    discovered: &mut HashMap<i64, LinkedEntityContext>,
    queued: &mut HashSet<i64>,
    queue: &mut VecDeque<(i64, usize)>,
) -> Result<()> {
    for (link, entity) in repo.list_outgoing_links(entity_id)? {
        record_link_context(discovered, root_entity, link, entity.clone(), depth);
        if queued.insert(entity.entity_id) {
            queue.push_back((entity.entity_id, depth));
        }
    }

    for (link, entity) in repo.list_incoming_links(entity_id)? {
        record_link_context(discovered, root_entity, link, entity.clone(), depth);
        if queued.insert(entity.entity_id) {
            queue.push_back((entity.entity_id, depth));
        }
    }

    Ok(())
}

fn record_link_context(
    discovered: &mut HashMap<i64, LinkedEntityContext>,
    root_entity: &Entity,
    link: EntityLink,
    related: Entity,
    depth: usize,
) {
    if related.entity_id == root_entity.entity_id {
        return;
    }

    let context = discovered.entry(related.entity_id).or_insert_with(|| LinkedEntityContext {
        link: link.clone(),
        entity: related.clone(),
        depth,
    });

    if depth < context.depth {
        context.link = link;
        context.entity = related;
        context.depth = depth;
        return;
    }

    if depth == context.depth {
        let current_edge = format!("{}->{}", context.link.from_entity_id, context.link.to_entity_id);
        let candidate_edge = format!("{}->{}", link.from_entity_id, link.to_entity_id);
        if candidate_edge < current_edge {
            context.link = link;
            context.entity = related;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::entity::{NewEntity, parse_lookup};

    fn setup_repo() -> EntityRepository<'static> {
        let conn = Box::leak(Box::new(db::open_in_memory().expect("db should open")));
        conn.execute_batch(db::schema_sql()).unwrap();
        EntityRepository::new(conn)
    }

    #[test]
    fn returns_empty_for_zero_depth() {
        let repo = setup_repo();
        let links = collect_link_context(&repo, 1, 0).unwrap();
        assert!(links.is_empty());
    }

    #[test]
    fn traverses_bidirectional_neighborhood_and_dedupes_root() {
        let repo = setup_repo();
        repo.create(&NewEntity { name: "student".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "subject".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "teacher".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "department".into(), r#ref: None, description: None }).unwrap();

        repo.link(&parse_lookup("student"), &parse_lookup("subject"), "student has subjects").unwrap();
        repo.link(&parse_lookup("teacher"), &parse_lookup("student"), "teacher mentors student").unwrap();
        repo.link(&parse_lookup("department"), &parse_lookup("teacher"), "department employs teacher").unwrap();

        let student = repo.find(&parse_lookup("student")).unwrap().unwrap();
        let linked = collect_link_context(&repo, student.entity_id, 2).unwrap();

        assert_eq!(linked.len(), 3);
        assert!(linked.iter().any(|item| item.entity.name == "subject" && item.depth == 1));
        assert!(linked.iter().any(|item| item.entity.name == "teacher" && item.depth == 1));
        assert!(linked.iter().any(|item| item.entity.name == "department" && item.depth == 2));
        assert!(!linked.iter().any(|item| item.entity.name == "student"));
    }

    #[test]
    fn prefers_shallower_path_then_lexical_edge_for_ties() {
        let repo = setup_repo();
        repo.create(&NewEntity { name: "root".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "alpha".into(), r#ref: None, description: None }).unwrap();
        repo.create(&NewEntity { name: "beta".into(), r#ref: None, description: None }).unwrap();

        repo.link(&parse_lookup("root"), &parse_lookup("alpha"), "root to alpha").unwrap();
        repo.link(&parse_lookup("beta"), &parse_lookup("root"), "beta to root").unwrap();
        repo.link(&parse_lookup("beta"), &parse_lookup("alpha"), "beta to alpha").unwrap();

        let alpha = repo.find(&parse_lookup("alpha")).unwrap().unwrap();
        let linked = collect_link_context(&repo, alpha.entity_id, 2).unwrap();
        let root = linked.iter().find(|item| item.entity.name == "root").unwrap();
        assert_eq!(root.depth, 1);
        assert_eq!((root.link.from_entity_id, root.link.to_entity_id), (1, 2));
    }
}
