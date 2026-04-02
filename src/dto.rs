use serde::{Deserialize, Serialize};

use crate::anchor::Anchor;
use crate::entity::Entity;
use crate::service::anchor_service::{AnchorShowResult, AnchorSyncResult};
use crate::service::entity_service::{
    EntityAutoResult, EntityContextResult, EntityShowResult, LinkedEntityContext,
};
use crate::service::health_service::HealthReport;

// (#!#tep:dto)
// [#!#tep:dto](dto)

/// Flat entity representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDto {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Flat anchor representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorDto {
    pub id: i64,
    pub name: String,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shift: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

/// A directional link in the entity graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDto {
    /// `"->"` (outgoing from root) or `"<-"` (incoming to root)
    pub direction: String,
    pub entity: EntityDto,
    pub relation: String,
    pub depth: usize,
}

// --- Response DTOs ---

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityShowDto {
    pub entity: EntityDto,
    pub anchors: Vec<AnchorDto>,
    pub links: Vec<LinkDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityContextDto {
    pub entity: EntityDto,
    pub anchors: Vec<AnchorContextDto>,
    pub links: Vec<LinkDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnchorContextDto {
    pub anchor: AnchorDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnchorShowDto {
    pub anchor: AnchorDto,
    pub entities: Vec<EntityDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityAutoDto {
    pub files_processed: usize,
    pub declarations_seen: usize,
    pub entities_ensured: usize,
    pub refs_filled: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnchorSyncDto {
    pub files_processed: usize,
    pub anchors_seen: usize,
    pub anchors_created: usize,
    pub anchors_dropped: usize,
    pub relations_synced: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthDto {
    pub files_scanned: usize,
    pub anchors_seen: usize,
    pub anchors_healthy: usize,
    pub anchors_moved: usize,
    pub anchors_missing: usize,
    pub duplicate_anchor_ids: usize,
    pub unknown_anchor_ids: usize,
    pub entities_without_anchors: usize,
    pub anchors_without_entities: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<String>,
}

// --- Conversions ---

pub fn entity_to_dto(e: &Entity) -> EntityDto {
    EntityDto {
        id: e.entity_id,
        name: e.name.clone(),
        r#ref: e.r#ref.clone(),
        description: e.description.clone(),
    }
}

pub fn anchor_to_dto(a: &Anchor) -> AnchorDto {
    AnchorDto {
        id: a.anchor_id,
        name: a.name.clone().unwrap_or_default(),
        file: a.file_path.clone(),
        line: a.line,
        shift: a.shift,
        offset: a.offset,
    }
}

fn link_to_dto(root_id: i64, item: &LinkedEntityContext) -> LinkDto {
    LinkDto {
        direction: if item.link.from_entity_id == root_id {
            "->".into()
        } else {
            "<-".into()
        },
        entity: entity_to_dto(&item.entity),
        relation: item.link.relation.clone(),
        depth: item.depth,
    }
}

pub fn entity_show_to_dto(result: &EntityShowResult) -> EntityShowDto {
    let root_id = result.entity.entity_id;
    EntityShowDto {
        entity: entity_to_dto(&result.entity),
        anchors: result.anchors.iter().map(anchor_to_dto).collect(),
        links: result
            .linked_entities
            .iter()
            .map(|l| link_to_dto(root_id, l))
            .collect(),
    }
}

pub fn entity_context_to_dto(result: &EntityContextResult) -> EntityContextDto {
    let root_id = result.entity.entity_id;
    EntityContextDto {
        entity: entity_to_dto(&result.entity),
        anchors: result
            .anchors
            .iter()
            .map(|a| AnchorContextDto {
                anchor: anchor_to_dto(&a.anchor),
                snippet: a.snippet.clone(),
            })
            .collect(),
        links: result
            .linked_entities
            .iter()
            .map(|l| link_to_dto(root_id, l))
            .collect(),
    }
}

pub fn anchor_show_to_dto(result: &AnchorShowResult) -> AnchorShowDto {
    AnchorShowDto {
        anchor: anchor_to_dto(&result.anchor),
        entities: result.entities.iter().map(entity_to_dto).collect(),
    }
}

pub fn anchor_sync_to_dto(result: &AnchorSyncResult) -> AnchorSyncDto {
    AnchorSyncDto {
        files_processed: result.files_processed,
        anchors_seen: result.anchors_seen,
        anchors_created: result.anchors_created,
        anchors_dropped: result.anchors_dropped,
        relations_synced: result.relations_synced,
    }
}

pub fn entity_auto_to_dto(result: &EntityAutoResult) -> EntityAutoDto {
    EntityAutoDto {
        files_processed: result.files_processed,
        declarations_seen: result.declarations_seen,
        entities_ensured: result.entities_ensured,
        refs_filled: result.refs_filled,
    }
}

pub fn health_to_dto(report: &HealthReport) -> HealthDto {
    let mut issues: Vec<String> = Vec::new();
    issues.extend_from_slice(&report.groups.moved_anchors);
    issues.extend_from_slice(&report.groups.missing_anchors);
    issues.extend_from_slice(&report.groups.duplicate_anchor_ids);
    issues.extend_from_slice(&report.groups.unknown_anchor_ids);
    issues.extend_from_slice(&report.groups.entities_without_anchors);
    issues.extend_from_slice(&report.groups.anchors_without_entities);
    HealthDto {
        files_scanned: report.files_scanned,
        anchors_seen: report.anchors_seen,
        anchors_healthy: report.anchors_healthy,
        anchors_moved: report.issue_counts.anchors_moved,
        anchors_missing: report.issue_counts.anchors_missing,
        duplicate_anchor_ids: report.issue_counts.duplicate_anchor_ids,
        unknown_anchor_ids: report.issue_counts.unknown_anchor_ids,
        entities_without_anchors: report.issue_counts.entities_without_anchors,
        anchors_without_entities: report.issue_counts.anchors_without_entities,
        issues,
    }
}

// #tepignoreafter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::Anchor;
    use crate::entity::{Entity, EntityLink};
    use crate::service::anchor_service::AnchorShowResult;
    use crate::service::entity_service::{
        EntityContextAnchor, EntityContextResult, EntityShowResult, LinkedEntityContext,
    };

    fn sample_entity() -> Entity {
        Entity {
            entity_id: 42,
            name: "auth.flow".into(),
            r#ref: Some("./src/auth.rs".into()),
            description: Some("Auth flow".into()),
            created_at: "1".into(),
            updated_at: "2".into(),
        }
    }

    fn sample_anchor() -> Anchor {
        Anchor {
            anchor_id: 7,
            version: 1,
            name: Some("auth.entry".into()),
            file_path: "./src/auth.rs".into(),
            line: Some(10),
            shift: Some(2),
            offset: Some(100),
            created_at: "1".into(),
            updated_at: "2".into(),
        }
    }

    fn sample_link(from: i64, to: i64, relation: &str, depth: usize) -> LinkedEntityContext {
        LinkedEntityContext {
            link: EntityLink {
                from_entity_id: from,
                to_entity_id: to,
                relation: relation.into(),
                created_at: "1".into(),
                updated_at: "2".into(),
            },
            entity: Entity {
                entity_id: to,
                name: "token".into(),
                r#ref: Some("./src/token.rs".into()),
                description: None,
                created_at: "1".into(),
                updated_at: "2".into(),
            },
            depth,
        }
    }

    #[test]
    fn entity_to_dto_maps_all_fields() {
        let dto = entity_to_dto(&sample_entity());
        assert_eq!(dto.id, 42);
        assert_eq!(dto.name, "auth.flow");
        assert_eq!(dto.r#ref.as_deref(), Some("./src/auth.rs"));
        assert_eq!(dto.description.as_deref(), Some("Auth flow"));
    }

    #[test]
    fn anchor_to_dto_maps_all_fields() {
        let dto = anchor_to_dto(&sample_anchor());
        assert_eq!(dto.id, 7);
        assert_eq!(dto.name, "auth.entry");
        assert_eq!(dto.file, "./src/auth.rs");
        assert_eq!(dto.line, Some(10));
        assert_eq!(dto.shift, Some(2));
        assert_eq!(dto.offset, Some(100));
    }

    #[test]
    fn entity_show_to_dto_maps_anchors_and_links() {
        let outgoing = sample_link(42, 10, "produces token", 1);
        let incoming = sample_link(5, 42, "caller uses auth", 1);

        let dto = entity_show_to_dto(&EntityShowResult {
            entity: sample_entity(),
            anchors: vec![sample_anchor()],
            linked_entities: vec![outgoing, incoming],
        });

        assert_eq!(dto.entity.id, 42);
        assert_eq!(dto.anchors.len(), 1);
        assert_eq!(dto.anchors[0].id, 7);
        assert_eq!(dto.links.len(), 2);
        assert!(
            dto.links
                .iter()
                .any(|l| l.direction == "->" && l.relation == "produces token")
        );
        assert!(
            dto.links
                .iter()
                .any(|l| l.direction == "<-" && l.relation == "caller uses auth")
        );
    }

    #[test]
    fn anchor_show_to_dto_maps_anchor_and_entities() {
        let dto = anchor_show_to_dto(&AnchorShowResult {
            anchor: sample_anchor(),
            entities: vec![sample_entity()],
        });
        assert_eq!(dto.anchor.id, 7);
        assert_eq!(dto.anchor.name, "auth.entry");
        assert_eq!(dto.entities.len(), 1);
        assert_eq!(dto.entities[0].id, 42);
        assert_eq!(dto.entities[0].name, "auth.flow");
    }

    #[test]
    fn entity_context_to_dto_includes_snippet_and_links() {
        let dto = entity_context_to_dto(&EntityContextResult {
            entity: sample_entity(),
            anchors: vec![EntityContextAnchor {
                anchor: sample_anchor(),
                snippet: Some("fn authenticate() {}".into()),
            }],
            linked_entities: vec![sample_link(42, 10, "produces token", 1)],
        });
        assert_eq!(dto.entity.id, 42);
        assert_eq!(dto.anchors.len(), 1);
        assert_eq!(
            dto.anchors[0].snippet.as_deref(),
            Some("fn authenticate() {}")
        );
        assert_eq!(dto.links.len(), 1);
        assert_eq!(dto.links[0].direction, "->");
        assert_eq!(dto.links[0].relation, "produces token");
    }

    #[test]
    fn link_direction_outgoing_when_from_is_root() {
        let link = sample_link(42, 10, "outgoing", 1);
        let dto = entity_show_to_dto(&EntityShowResult {
            entity: sample_entity(),
            anchors: vec![],
            linked_entities: vec![link],
        });
        assert_eq!(dto.links[0].direction, "->");
    }

    #[test]
    fn link_direction_incoming_when_to_is_root() {
        let link = sample_link(99, 42, "incoming", 1);
        let dto = entity_show_to_dto(&EntityShowResult {
            entity: sample_entity(),
            anchors: vec![],
            linked_entities: vec![link],
        });
        assert_eq!(dto.links[0].direction, "<-");
    }
}
