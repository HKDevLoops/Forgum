//! `forgum showcase` — a scripted 60-second demo reel for presentations.

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ShowcaseSegment {
    pub name: &'static str,
    pub duration_secs: u64,
    pub description: &'static str,
}

pub fn segments() -> Vec<ShowcaseSegment> {
    vec![
        ShowcaseSegment {
            name: "portal materialize",
            duration_secs: 5,
            description: "Cow materializes through a portal effect",
        },
        ShowcaseSegment {
            name: "aurora reactive",
            duration_secs: 10,
            description: "Aurora with audio-reactive frequency bands",
        },
        ShowcaseSegment {
            name: "shatter and reassemble",
            duration_secs: 5,
            description: "Cow shatters and reassembles with a greeting",
        },
        ShowcaseSegment {
            name: "cpu-reactive ember",
            duration_secs: 10,
            description: "Ember effect intensity driven by CPU load",
        },
        ShowcaseSegment {
            name: "battle joust",
            duration_secs: 10,
            description: "Two cows charge and joust",
        },
        ShowcaseSegment {
            name: "herd dashboard",
            duration_secs: 10,
            description: "Live herd monitoring dashboard",
        },
        ShowcaseSegment {
            name: "seasonal reveal",
            duration_secs: 10,
            description: "Current seasonal theme applied automatically",
        },
    ]
}

pub fn run_showcase() -> String {
    let segs = segments();
    let total: u64 = segs.iter().map(|s| s.duration_secs).sum();
    let start = Instant::now();

    let mut output = String::new();
    output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
    output.push_str("║                    FORGUM SHOWCASE                          ║\n");
    output.push_str("║              The Terminal Cow Companion                     ║\n");
    output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

    for (i, seg) in segs.iter().enumerate() {
        let elapsed = start.elapsed().as_secs();
        let progress = (elapsed as f64 / total as f64 * 100.0) as u32;

        output.push_str(&format!(
            "\r\x1b[2K[{:>3}%] Segment {}/{}: {} — {}",
            progress,
            i + 1,
            segs.len(),
            seg.name,
            seg.description,
        ));
        output.push('\n');

        let segment_start = Instant::now();
        while segment_start.elapsed() < Duration::from_secs(seg.duration_secs) {
            let sub_elapsed = segment_start.elapsed().as_millis() as f32;
            let sub_total = seg.duration_secs as f32 * 1000.0;
            let sub_progress = (sub_elapsed / sub_total * 100.0) as u32;

            output.push_str(&format!(
                "\r  [{}{}] {}%",
                "█".repeat(sub_progress as usize / 2),
                "░".repeat(50 - sub_progress as usize / 2),
                sub_progress,
            ));

            std::thread::sleep(Duration::from_millis(100));
        }
        output.push('\n');
    }

    let total_secs = start.elapsed().as_secs_f64();
    output.push_str(&format!(
        "\nShowcase complete in {:.1}s. Forgum makes your terminal alive!\n",
        total_secs
    ));

    output
}

pub fn render_showcase_frame(segment: &ShowcaseSegment, progress: f32) -> String {
    let bar_width = 40;
    let clamped = progress.clamp(0.0, 1.0);
    let filled = (clamped * bar_width as f32) as usize;
    let empty = bar_width - filled;

    format!(
        "  {} {} [{}{}]",
        segment.name,
        segment.description,
        "█".repeat(filled),
        "░".repeat(empty),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_count() {
        assert_eq!(segments().len(), 7);
    }

    #[test]
    fn total_duration() {
        let total: u64 = segments().iter().map(|s| s.duration_secs).sum();
        assert_eq!(total, 60);
    }

    #[test]
    fn segments_have_valid_fields() {
        for seg in segments() {
            assert!(!seg.name.is_empty());
            assert!(!seg.description.is_empty());
            assert!(seg.duration_secs > 0);
        }
    }

    #[test]
    fn render_frame_at_zero_progress() {
        let seg = &segments()[0];
        let frame = render_showcase_frame(seg, 0.0);
        assert!(frame.contains(seg.name));
        assert!(frame.contains("░"));
        assert!(!frame.contains("█"));
    }

    #[test]
    fn render_frame_at_full_progress() {
        let seg = &segments()[0];
        let frame = render_showcase_frame(seg, 1.0);
        assert!(frame.contains("█"));
    }

    #[test]
    fn render_frame_at_half_progress() {
        let seg = &segments()[0];
        let frame = render_showcase_frame(seg, 0.5);
        assert!(frame.contains("█"));
        assert!(frame.contains("░"));
    }

    #[test]
    fn render_frame_out_of_bounds_clamps() {
        let seg = &segments()[0];
        let over = render_showcase_frame(seg, 2.0);
        let under = render_showcase_frame(seg, -1.0);
        let full = render_showcase_frame(seg, 1.0);
        assert_eq!(over, full, "progress > 1.0 should clamp to full bar");
        assert_eq!(
            under,
            render_showcase_frame(seg, 0.0),
            "progress < 0.0 should clamp to empty bar"
        );
    }

    #[test]
    fn render_frame_negative_progress() {
        let seg = &segments()[0];
        let frame = render_showcase_frame(seg, -0.5);
        let zero_frame = render_showcase_frame(seg, 0.0);
        assert_eq!(frame, zero_frame);
        assert!(!frame.contains("█"));
    }

    #[test]
    fn render_frame_large_progress() {
        let seg = &segments()[0];
        let frame = render_showcase_frame(seg, 100.0);
        let full_frame = render_showcase_frame(seg, 1.0);
        assert_eq!(frame, full_frame);
        assert!(frame.contains("█"));
    }

    #[test]
    fn segments_names_are_unique() {
        let segs = segments();
        let mut names: Vec<&str> = segs.iter().map(|s| s.name).collect();
        let original_len = names.len();
        names.dedup();
        assert_eq!(names.len(), original_len, "segment names must be unique");
    }

    #[test]
    fn segments_descriptions_are_nonempty() {
        for seg in segments() {
            assert!(!seg.description.is_empty());
            assert!(
                seg.description.len() > 5,
                "description '{}' is too short ({} chars, need > 5)",
                seg.description,
                seg.description.len(),
            );
        }
    }

    #[test]
    #[ignore]
    fn run_showcase_returns_string() {
        let output = run_showcase();
        assert!(output.contains("SHOWCASE"));
    }
}
