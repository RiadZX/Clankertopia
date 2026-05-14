use std::collections::HashSet;

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::office::OfficeConfigRes;

pub struct ClankerCommunicationPlugin;

impl Plugin for ClankerCommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClankerCommunicationState>()
            .add_message::<PromptSubmitted>()
            .add_systems(
                Update,
                (
                    toggle_discussion_mode_system,
                    handle_prompt_submission_system,
                    advance_discussion_system,
                ),
            );
    }
}

#[derive(Message, Debug, Clone)]
pub struct PromptSubmitted {
    pub source: Entity,
    pub prompt: String,
}

#[derive(Resource, Debug, Clone)]
pub struct ClankerCommunicationState {
    pub discussion_enabled: bool,
    pub banner: String,
    pub latest_verdict: Option<DiscussionVerdict>,
    active_timer_secs: f32,
}

impl Default for ClankerCommunicationState {
    fn default() -> Self {
        Self {
            discussion_enabled: false,
            banner: "Discussion: OFF (press F7 to toggle)".to_string(),
            latest_verdict: None,
            active_timer_secs: 0.0,
        }
    }
}

#[derive(Component)]
struct MeetingVisual;

fn toggle_discussion_mode_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ClankerCommunicationState>,
) {
    if keys.just_pressed(KeyCode::F7) {
        state.discussion_enabled = !state.discussion_enabled;
        state.banner = if state.discussion_enabled {
            "Discussion: ON (F7 to disable)".to_string()
        } else {
            "Discussion: OFF (press F7 to toggle)".to_string()
        };
        if !state.discussion_enabled {
            state.active_timer_secs = 0.0;
        }
    }
}

fn handle_prompt_submission_system(
    mut commands: Commands,
    mut prompts: MessageReader<PromptSubmitted>,
    cfg: Res<OfficeConfigRes>,
    mut state: ResMut<ClankerCommunicationState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_visuals: Query<Entity, With<MeetingVisual>>,
) {
    for prompt_event in prompts.read() {
        if !state.discussion_enabled {
            continue;
        }
        let _source = prompt_event.source;
        let clankers = collect_clankers(&cfg.0);
        if clankers.is_empty() {
            continue;
        }
        let verdict = run_discussion(&prompt_event.prompt, &clankers);
        state.banner = format!(
            "Negotiating '{}' ({} clankers)...",
            truncate_for_banner(&prompt_event.prompt),
            clankers.len()
        );
        state.active_timer_secs = 8.0;
        state.latest_verdict = Some(verdict);

        for e in existing_visuals.iter() {
            commands.entity(e).despawn();
        }
        spawn_meeting_visuals(
            &mut commands,
            meshes.as_mut(),
            materials.as_mut(),
            &clankers,
        );
    }
}

fn advance_discussion_system(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<ClankerCommunicationState>,
    visuals: Query<Entity, With<MeetingVisual>>,
) {
    if state.active_timer_secs <= 0.0 {
        return;
    }
    state.active_timer_secs -= time.delta_secs();
    if state.active_timer_secs > 5.0 {
        state.banner = "Discussion phase: sharing proposals".to_string();
        return;
    }
    if state.active_timer_secs > 2.0 {
        state.banner = "Discussion phase: scoring and negotiation".to_string();
        return;
    }
    if let Some(verdict) = &state.latest_verdict {
        state.banner = format!(
            "Final verdict: {} ({} pts) — {}",
            verdict.winner, verdict.winning_score, verdict.winning_summary
        );
    }
    if state.active_timer_secs <= 0.0 {
        for e in visuals.iter() {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_meeting_visuals(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    clankers: &[ClankerProfile],
) {
    let center = Vec3::new(0.0, 0.0, 0.0);
    let table_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.22, 0.16),
        perceptual_roughness: 0.78,
        ..default()
    });
    let table_top = meshes.add(Cuboid::new(2.5, 0.12, 2.5));
    commands.spawn((
        MeetingVisual,
        Mesh3d(table_top),
        MeshMaterial3d(table_mat.clone()),
        Transform::from_translation(center + Vec3::new(0.0, 0.85, 0.0)),
    ));
    let table_leg = meshes.add(Cylinder::new(0.2, 0.85));
    commands.spawn((
        MeetingVisual,
        Mesh3d(table_leg),
        MeshMaterial3d(table_mat),
        Transform::from_translation(center + Vec3::new(0.0, 0.425, 0.0)),
    ));

    let participant_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.7, 0.95),
        emissive: LinearRgba::new(0.05, 0.08, 0.12, 1.0),
        ..default()
    });
    let avatar_mesh = meshes.add(Capsule3d::new(0.12, 0.45));
    let n = clankers.len().max(1) as f32;
    for (i, clanker) in clankers.iter().enumerate() {
        let angle = std::f32::consts::TAU * (i as f32) / n;
        let seat_pos = center + Vec3::new(angle.cos() * 1.45, 0.35, angle.sin() * 1.45);
        commands.spawn((
            MeetingVisual,
            Mesh3d(avatar_mesh.clone()),
            MeshMaterial3d(participant_mat.clone()),
            Transform::from_translation(seat_pos).with_rotation(Quat::from_axis_angle(
                Vec3::Y,
                -angle + std::f32::consts::PI,
            )),
        ));
        commands.spawn((
            MeetingVisual,
            Text2d::new(clanker.name.clone()),
            TextFont {
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.98, 1.0)),
            Transform::from_translation(seat_pos + Vec3::new(0.0, 0.65, 0.0)),
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptDomain {
    Coding,
    Research,
    Writing,
    Ops,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Specialty {
    Coding,
    Research,
    Writing,
    Ops,
    General,
}

#[derive(Debug, Clone)]
struct ClankerProfile {
    name: String,
    specialty: Specialty,
}

#[derive(Debug, Clone)]
pub struct DiscussionVerdict {
    pub winner: String,
    pub winning_score: i32,
    pub winning_summary: String,
    pub scoreboard: Vec<(String, i32)>,
}

fn collect_clankers(cfg: &crate::office::config::OfficeConfig) -> Vec<ClankerProfile> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for room in &cfg.rooms {
        for desk in &room.desks {
            let name = desk
                .name
                .clone()
                .unwrap_or_else(|| format!("Clanker {}", out.len() + 1));
            if !seen.insert(name.clone()) {
                continue;
            }
            out.push(ClankerProfile {
                specialty: infer_specialty(&name),
                name,
            });
        }
    }
    out
}

fn infer_specialty(name: &str) -> Specialty {
    let lower = name.to_lowercase();
    if lower.contains("claude") || lower.contains("code") || lower.contains("dev") {
        Specialty::Coding
    } else if lower.contains("research") || lower.contains("gemini") || lower.contains("deep") {
        Specialty::Research
    } else if lower.contains("writer") || lower.contains("copy") {
        Specialty::Writing
    } else if lower.contains("ops") || lower.contains("infra") {
        Specialty::Ops
    } else {
        Specialty::General
    }
}

fn infer_domain(prompt: &str) -> PromptDomain {
    let p = prompt.to_lowercase();
    if ["code", "rust", "bug", "compile", "test", "refactor"]
        .iter()
        .any(|k| p.contains(k))
    {
        PromptDomain::Coding
    } else if ["research", "compare", "analyze", "investigate"]
        .iter()
        .any(|k| p.contains(k))
    {
        PromptDomain::Research
    } else if ["write", "blog", "story", "copy", "email"]
        .iter()
        .any(|k| p.contains(k))
    {
        PromptDomain::Writing
    } else if ["deploy", "monitor", "incident", "ops", "infra"]
        .iter()
        .any(|k| p.contains(k))
    {
        PromptDomain::Ops
    } else {
        PromptDomain::General
    }
}

fn score_candidate(prompt: &str, domain: PromptDomain, profile: &ClankerProfile) -> i32 {
    let mut score = 40 + deterministic_jitter(prompt, &profile.name);
    if specialty_matches_domain(profile.specialty, domain) {
        score += 35;
    }
    let prompt_tokens = tokens(prompt);
    let specialty_tokens = tokens(specialty_label(profile.specialty));
    let overlap = prompt_tokens
        .iter()
        .filter(|t| specialty_tokens.contains(*t))
        .count() as i32;
    score + overlap * 8
}

fn specialty_matches_domain(s: Specialty, d: PromptDomain) -> bool {
    matches!(
        (s, d),
        (Specialty::Coding, PromptDomain::Coding)
            | (Specialty::Research, PromptDomain::Research)
            | (Specialty::Writing, PromptDomain::Writing)
            | (Specialty::Ops, PromptDomain::Ops)
            | (Specialty::General, PromptDomain::General)
    )
}

fn specialty_label(s: Specialty) -> &'static str {
    match s {
        Specialty::Coding => "coding rust tests implementation",
        Specialty::Research => "research analysis evidence comparison",
        Specialty::Writing => "writing structure clarity communication",
        Specialty::Ops => "ops deployment reliability monitoring",
        Specialty::General => "general planning synthesis",
    }
}

fn deterministic_jitter(a: &str, b: &str) -> i32 {
    let mut hash: u32 = 2_166_136_261;
    for byte in a.bytes().chain(b.bytes()) {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    (hash % 17) as i32
}

fn run_discussion(prompt: &str, clankers: &[ClankerProfile]) -> DiscussionVerdict {
    let domain = infer_domain(prompt);
    let mut candidates: Vec<(String, i32)> = clankers
        .iter()
        .map(|c| (c.name.clone(), score_candidate(prompt, domain, c)))
        .collect();
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut leader_name = candidates
        .first()
        .map(|(n, _)| n.clone())
        .unwrap_or_else(|| "Clanker".to_string());
    let mut leader_score = candidates.first().map(|(_, s)| *s).unwrap_or(0);
    let mut negotiated_boost = 0_i32;
    for (name, score) in candidates.iter().skip(1) {
        let pressure = ((*score - leader_score) / 8).clamp(-3, 3);
        negotiated_boost += pressure;
        if specialty_matches_domain(infer_specialty(name), domain) {
            negotiated_boost += 1;
        }
    }
    leader_score += negotiated_boost;
    if let Some((top_name, top_score)) = candidates.first_mut() {
        *top_name = leader_name.clone();
        *top_score = leader_score;
    }
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    if let Some((winner, score)) = candidates.first() {
        leader_name = winner.clone();
        leader_score = *score;
    }

    let negotiated_summary = format!(
        "{} led a {:?} specialist-first round, then refined the proposal with peer scoring from {} clankers.",
        leader_name,
        domain,
        clankers.len()
    );
    DiscussionVerdict {
        winner: leader_name,
        winning_score: leader_score,
        winning_summary: negotiated_summary,
        scoreboard: candidates,
    }
}

fn truncate_for_banner(prompt: &str) -> String {
    let trimmed = prompt.trim();
    if trimmed.chars().count() <= 40 {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(40).collect();
    out.push('…');
    out
}

fn tokens(input: &str) -> Vec<String> {
    input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profiles() -> Vec<ClankerProfile> {
        vec![
            ClankerProfile {
                name: "Claude Coder".to_string(),
                specialty: Specialty::Coding,
            },
            ClankerProfile {
                name: "Gemini Research".to_string(),
                specialty: Specialty::Research,
            },
            ClankerProfile {
                name: "Ops Bot".to_string(),
                specialty: Specialty::Ops,
            },
        ]
    }

    #[test]
    fn coding_prompt_prefers_coding_specialist() {
        let verdict = run_discussion("Refactor this Rust code and add tests", &profiles());
        assert_eq!(verdict.winner, "Claude Coder");
        assert!(verdict.winning_score >= verdict.scoreboard[1].1);
    }

    #[test]
    fn produces_ranked_scoreboard_and_summary() {
        let verdict = run_discussion(
            "Investigate failure trends and summarize findings",
            &profiles(),
        );
        assert_eq!(verdict.scoreboard.len(), 3);
        assert!(verdict.scoreboard[0].1 >= verdict.scoreboard[1].1);
        assert!(verdict.winning_summary.contains("specialist-first"));
    }
}
