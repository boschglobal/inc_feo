// Copyright 2025 SCORE project
//
// SPDX-License-Identifier: Apache-2.0

use crate::activities::components::{AsciiArtGenerator, AsciiArtPrinter};
use crate::activities::messages::AsciiArt;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use feo::activity::{ActivityBuilder, ActivityIdAndBuilder};
use feo::ids::{ActivityId, AgentId, WorkerId};
use feo::topicspec::{Direction, TopicSpecification};
use feo_com::interface::ComBackend;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type WorkerAssignment = (WorkerId, Vec<(ActivityId, Box<dyn ActivityBuilder>)>);

// For each activity, list the activities it needs to wait for
pub type ActivityDependencies = HashMap<ActivityId, Vec<ActivityId>>;

#[cfg(feature = "com_iox2")]
pub const COM_BACKEND: ComBackend = ComBackend::Iox2;
#[cfg(feature = "com_linux_shm")]
pub const COM_BACKEND: ComBackend = ComBackend::LinuxShm;

pub const BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);
pub const BIND_ADDR2: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082);

//  Input string to for FEO Ascii Art activities

pub const DEFAULT_INPUT_STRING: &str = "Honda forever!";
// Number of lines in ASCII art (each character is 8 lines tall for big.flf font)
pub const NUM_LINES: usize = 8;

// Topic prefix
pub const TOPIC_PREFIX: &str = "feo/com/ascii_name";

// Max additional subscribers
pub const MAX_ADDITIONAL_SUBSCRIBERS: usize = 2;

// Define Activity IDs
pub const ACTIVITY_GENERATOR: ActivityId = ActivityId::new(0);

// Worker ID
pub const WORKER_ASCII_ART: WorkerId = WorkerId::new(40);

// Agent ID
pub const AGENT_PRIMARY: AgentId = AgentId::new(100);

pub fn socket_paths() -> (PathBuf, PathBuf) {
    (
        Path::new("/tmp/feo_listener1.socket").to_owned(),
        Path::new("/tmp/feo_listener2.socket").to_owned(),
    )
}

pub fn agent_workerpool_assignments() -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityIdAndBuilder>)>>
{
    // Create all activities
    let mut activities = Vec::new();

    // Add generator activity (first activity)
    activities.push(create_ascii_art_generator_activity());

    // Add printer activities (one for each line of ASCII art)
    for i in 0..NUM_LINES {
        activities.push(create_ascii_art_printer_activity(i));
    }

    let ascii_art_activity_to_worker_assignment =
        create_activity_to_worker_assignment(WORKER_ASCII_ART, activities);

    #[cfg(any(
        feature = "signalling_direct_tcp",
        feature = "signalling_direct_unix",
        feature = "signalling_relayed_tcp",
        feature = "signalling_relayed_unix"
    ))]
    let agent_workerpool_assigments = [create_agent_workerpool_assignment(
        AGENT_PRIMARY,
        vec![ascii_art_activity_to_worker_assignment],
    )]
    .into_iter()
    .collect();

    #[cfg(feature = "signalling_direct_mpsc")]
    let agent_workerpool_assigments = [create_agent_workerpool_assignment(
        AGENT_PRIMARY,
        vec![ascii_art_activity_to_worker_assignment],
    )]
    .into_iter()
    .collect();

    agent_workerpool_assigments
}

pub fn activity_dependencies() -> ActivityDependencies {
    let mut dependencies = HashMap::new();

    // Generator activity has no dependencies
    activity_depends_on(&mut dependencies, ACTIVITY_GENERATOR, None);

    // First printer depends on generator
    activity_depends_on(
        &mut dependencies,
        ActivityId::new(1),
        Some(ACTIVITY_GENERATOR),
    );

    // Each subsequent printer depends on the previous printer
    for i in 2..=NUM_LINES {
        activity_depends_on(
            &mut dependencies,
            ActivityId::new(i as u64),
            Some(ActivityId::new((i - 1) as u64)),
        );
    }

    dependencies
}

pub fn topic_dependencies<'a>() -> Vec<TopicSpecification<'a>> {
    let mut topics = Vec::new();

    // Static strings for topic names
    static GENERATOR_TOPIC: &str = "feo/com/ascii_name/generator_to_printer_0";

    // Generator to first printer
    topics.push(TopicSpecification::new::<AsciiArt>(
        GENERATOR_TOPIC,
        vec![
            writes_topic(ACTIVITY_GENERATOR),
            reads_topic(ActivityId::new(1)),
        ],
    ));

    // Between printers - using string literals that live long enough
    for i in 1..NUM_LINES {
        let current_activity_id = ActivityId::new((i) as u64);
        let next_activity_id = ActivityId::new((i + 1) as u64);

        // Use static topic names using a more dynamic approach
        // We need to use static strings for topic names
        let topic_str = match i {
            1 => "feo/com/ascii_name/printer_0_to_1",
            2 => "feo/com/ascii_name/printer_1_to_2",
            3 => "feo/com/ascii_name/printer_2_to_3",
            4 => "feo/com/ascii_name/printer_3_to_4",
            5 => "feo/com/ascii_name/printer_4_to_5",
            6 => "feo/com/ascii_name/printer_5_to_6",
            7 => "feo/com/ascii_name/printer_6_to_7",
            8 => "feo/com/ascii_name/printer_7_to_8",
            9 => "feo/com/ascii_name/printer_8_to_9",
            _ => "feo/com/ascii_name/printer_unknown",
        };

        topics.push(TopicSpecification::new::<AsciiArt>(
            topic_str,
            vec![
                writes_topic(current_activity_id),
                reads_topic(next_activity_id),
            ],
        ));
    }

    topics
}

pub fn worker_agent_map() -> HashMap<WorkerId, AgentId> {
    agent_workerpool_assignments()
        .iter()
        .flat_map(|(agent_id, worker_assignments)| {
            worker_assignments
                .iter()
                .map(move |(worker_id, _)| (*worker_id, *agent_id))
        })
        .collect()
}

pub fn agent_assignments_ids() -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityId>)>> {
    agent_workerpool_assignments()
        .into_iter()
        .map(|(agent_id, worker_assignments)| {
            (
                agent_id,
                worker_assignments
                    .into_iter()
                    .map(|(worker_id, activities)| {
                        (
                            worker_id,
                            activities
                                .into_iter()
                                .map(|(activity_id, _)| activity_id)
                                .collect(),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

/// Helper function to create an activity with its builder
fn create_activity(
    activity_id: ActivityId,
    builder: Box<dyn ActivityBuilder>,
) -> (ActivityId, Box<dyn ActivityBuilder>) {
    (activity_id, builder)
}

/// Helper function to create the ASCII art generator activity (first activity)
fn create_ascii_art_generator_activity() -> (ActivityId, Box<dyn ActivityBuilder>) {
    create_activity(
        ACTIVITY_GENERATOR,
        Box::new(move |id| {
            AsciiArtGenerator::build(
                id,
                DEFAULT_INPUT_STRING.to_string(),
                &format!("{TOPIC_PREFIX}/generator_to_printer_0"),
            )
        }),
    )
}

/// Helper function to create an ASCII art printer activity
fn create_ascii_art_printer_activity(line_index: usize) -> (ActivityId, Box<dyn ActivityBuilder>) {
    let activity_id = ActivityId::new((line_index + 1) as u64); // +1 because 0 is the generator

    // Input topic comes from previous activity
    let input_topic = if line_index == 0 {
        format!("{TOPIC_PREFIX}/generator_to_printer_0")
    } else {
        format!(
            "{}/printer_{}_to_{}",
            TOPIC_PREFIX,
            line_index - 1,
            line_index
        )
    };

    // Output topic (none for the last printer)
    let output_topic = if line_index < NUM_LINES - 1 {
        Some(format!(
            "{}/printer_{}_to_{}",
            TOPIC_PREFIX,
            line_index,
            line_index + 1
        ))
    } else {
        None
    };

    create_activity(
        activity_id,
        Box::new(move |id| {
            AsciiArtPrinter::build(id, line_index, &input_topic, output_topic.as_deref())
        }),
    )
}

/// Helper function to assign an activity to a worker with optional agent
fn create_activity_to_worker_assignment(
    worker_id: WorkerId,
    activities: Vec<(ActivityId, Box<dyn ActivityBuilder>)>,
) -> WorkerAssignment {
    (worker_id, activities)
}

/// Helper function to create an agent assignment
fn create_agent_workerpool_assignment(
    agent_id: AgentId,
    worker_assignments: Vec<WorkerAssignment>,
) -> (AgentId, Vec<WorkerAssignment>) {
    (agent_id, worker_assignments)
}

fn activity_depends_on(
    dependencies: &mut ActivityDependencies,
    activity: ActivityId,
    required_activity: Option<ActivityId>,
) {
    dependencies.entry(activity).or_default();
    if let Some(prereq) = required_activity {
        if let Some(deps) = dependencies.get_mut(&activity) {
            if !deps.contains(&prereq) {
                deps.push(prereq);
            }
        }
    }
}

/// Helper function to mark an activity as a reader of a topic
fn reads_topic(activity_id: ActivityId) -> (ActivityId, Direction) {
    (activity_id, Direction::Incoming)
}

/// Helper function to mark an activity as a writer to a topic
fn writes_topic(activity_id: ActivityId) -> (ActivityId, Direction) {
    (activity_id, Direction::Outgoing)
}
