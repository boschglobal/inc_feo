// Copyright 2025 Accenture.
//
// SPDX-License-Identifier: Apache-2.0

use crate::activities::messages::AsciiArt;
use crate::activities::ascii_art::get_ascii_art_char;
use core::ops::Deref;
use feo::activity::Activity;
use feo::ids::ActivityId;
use feo_com::interface::{ActivityInput, ActivityOutput};
#[cfg(feature = "com_iox2")]
use feo_com::iox2::{Iox2Input, Iox2Output};
#[cfg(feature = "com_linux_shm")]
use feo_com::linux_shm::{LinuxShmInput, LinuxShmOutput};
use feo_log::{debug, info};
use std::fmt;

/// Create an activity output.
fn activity_output<T>(topic: &str) -> Box<dyn ActivityOutput<T>>
where
    T: fmt::Debug + 'static,
{
    #[cfg(feature = "com_iox2")]
    return Box::new(Iox2Output::new(topic));
    #[cfg(feature = "com_linux_shm")]
    return Box::new(LinuxShmOutput::new(topic));
}

/// Create an activity input.
fn activity_input<T>(topic: &str) -> Box<dyn ActivityInput<T>>
where
    T: fmt::Debug + 'static,
{
    #[cfg(feature = "com_iox2")]
    return Box::new(Iox2Input::new(topic));
    #[cfg(feature = "com_linux_shm")]
    return Box::new(LinuxShmInput::new(topic));
}

/// AsciiArtGenerator Activity
///
/// First activity that generates the initial ASCII art
#[derive(Debug)]
pub struct AsciiArtGenerator {
    activity_id: ActivityId,
    input_string: String,
    output: Box<dyn ActivityOutput<AsciiArt>>,
}

impl AsciiArtGenerator {
    pub fn build(
        activity_id: ActivityId,
        input_string: String,
        output_topic: &str,
    ) -> Box<dyn Activity> {
        Box::new(Self {
            activity_id,
            input_string,
            output: activity_output::<AsciiArt>(output_topic),
        })
    }
    
    /// Generate ASCII art based on the input string
    fn generate_ascii_art(&self) -> Vec<String> {
        // Characters are 7 lines tall
        let mut result = vec![String::new(); 7];
        
        for c in self.input_string.chars() {
            // Get ASCII art for this character
            let char_art = get_ascii_art_char(c);
            
            // Append each line to the result
            for (i, line) in char_art.iter().enumerate() {
                if i < result.len() {
                    result[i].push_str(line);
                }
            }
        }
        
        result
    }
}

impl Activity for AsciiArtGenerator {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    fn startup(&mut self) {
        debug!("Starting AsciiArtGenerator for string: {}", self.input_string);
    }

    fn step(&mut self) {
        debug!("Generating ASCII art for: {}", self.input_string);
        
        // Create ASCII art message
        let ascii_art = AsciiArt {
            lines: self.generate_ascii_art(),
        };

        // Send ASCII art to next activity
        match self.output.write_uninit() {
            Ok(writer) => {
                let writer = writer.write_payload(ascii_art);
                if let Err(e) = writer.send() {
                    debug!("Error sending ASCII art: {:?}", e);
                } else {
                    debug!("Sent ASCII art to next activity");
                }
            }
            Err(e) => {
                debug!("Error preparing to write ASCII art: {:?}", e);
            }
        }
    }

    fn shutdown(&mut self) {
        debug!("Shutting down AsciiArtGenerator");
    }
}

/// AsciiArtPrinter Activity
///
/// Activity that prints a single specific line of the ASCII art
#[derive(Debug)]
pub struct AsciiArtPrinter {
    activity_id: ActivityId,
    line_index: usize,  // The specific line to print (just this one line)
    input: Box<dyn ActivityInput<AsciiArt>>,
    output: Option<Box<dyn ActivityOutput<AsciiArt>>>,
}

impl AsciiArtPrinter {
    pub fn build(
        activity_id: ActivityId,
        line_index: usize,
        input_topic: &str,
        output_topic: Option<&str>,
    ) -> Box<dyn Activity> {
        let output = output_topic.map(|topic| activity_output::<AsciiArt>(topic));
        
        Box::new(Self {
            activity_id,
            line_index,
            input: activity_input::<AsciiArt>(input_topic),
            output,
        })
    }
}

impl Activity for AsciiArtPrinter {
    fn id(&self) -> ActivityId {
        self.activity_id
    }

    //#[instrument(name = "AsciiArtPrinter startup")]
    fn startup(&mut self) {
        debug!("Starting AsciiArtPrinter for line {}", self.line_index);
    }

    fn step(&mut self) {
        debug!("Processing ASCII art for line {}", self.line_index);
        
        // Read the ASCII art from previous activity
        match self.input.read() {
            Ok(ascii_art_ref) => {
                let ascii_art = ascii_art_ref.deref();
                
                // Print only this specific line
                if self.line_index < ascii_art.lines.len() {
                    // Sleep for 100ms before printing
                    //std::thread::sleep(std::time::Duration::from_millis(100));
                    info!("Activity {} printing line {}: {}", self.activity_id, self.line_index, ascii_art.lines[self.line_index]);
                }
                
                // Forward the ASCII art to the next activity if needed
                if let Some(ref mut output) = self.output {
                    match output.write_uninit() {
                        Ok(writer) => {
                            let writer = writer.write_payload(ascii_art.clone());
                            if let Err(e) = writer.send() {
                                debug!("Error forwarding ASCII art: {:?}", e);
                            } else {
                                debug!("Forwarded ASCII art to next activity");
                            }
                        }
                        Err(e) => {
                            debug!("Error preparing to forward ASCII art: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                debug!("Error reading ASCII art: {:?}", e);
            }
        }
    }

    //#[instrument(name = "AsciiArtPrinter shutdown")]
    fn shutdown(&mut self) {
        debug!("Shutting down AsciiArtPrinter for line {}", self.line_index);
    }
}
