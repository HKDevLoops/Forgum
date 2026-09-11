//! Comprehensive test suite for Image to ASCII converter and scene mascot integration.

use forgum_engine::cli::{parse_args, Command, Commands};
use forgum_engine::cow;
use forgum_engine::image_ascii::{
    calculate_dimensions, convert_image_path_to_ascii, convert_image_to_ascii, image_to_cow,
    image_to_cow_from_image, save_custom_cow, AsciiConverterConfig, ColorMode, LuminanceRamp,
};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;
use std::sync::Mutex;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Helper to generate in-memory test images with distinct quadrant colors and alpha.
fn generate_quadrant_image(width: u32, height: u32) -> DynamicImage {
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        if x < width / 2 && y < height / 2 {
            // Top-left: bright white opaque (highest luminance)
            *pixel = Rgba([255, 255, 255, 255]);
        } else if x >= width / 2 && y < height / 2 {
            // Top-right: vivid blue
            *pixel = Rgba([0, 0, 255, 255]);
        } else if x < width / 2 && y >= height / 2 {
            // Bottom-left: vivid red
            *pixel = Rgba([255, 0, 0, 255]);
        } else {
            // Bottom-right: completely transparent
            *pixel = Rgba([0, 0, 0, 0]);
        }
    }
    DynamicImage::ImageRgba8(img)
}

/// Helper to generate encoded bytes in PNG or BMP format in memory.
fn encode_image(img: &DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, format)
        .expect("encode image in memory");
    buf.into_inner()
}

#[test]
fn test_aspect_ratio_preservation() {
    // 100x100 square with target width 40 -> height should scale to 20
    // because terminal characters have ~1:2 aspect ratio.
    let (w, h) = calculate_dimensions(100, 100, Some(40), None);
    assert_eq!(w, 40);
    assert_eq!(
        h, 20,
        "1:2 monospace font correction must yield h=20 for w=40 on square"
    );

    // Reverse: target height 20 -> width should scale to 40
    let (w2, h2) = calculate_dimensions(100, 100, None, Some(20));
    assert_eq!(w2, 40);
    assert_eq!(h2, 20);

    // Explicit dimensions override aspect ratio calculation
    let (w3, h3) = calculate_dimensions(100, 100, Some(35), Some(15));
    assert_eq!(w3, 35);
    assert_eq!(h3, 15);

    // Default dimensions (~40 width)
    let (w_def, h_def) = calculate_dimensions(100, 100, None, None);
    assert_eq!(w_def, 40);
    assert_eq!(h_def, 20);
}

#[test]
fn test_ascii_conversion_png_and_bmp_in_memory() {
    let original = generate_quadrant_image(64, 64);

    // 1. Test PNG decode from memory
    let png_bytes = encode_image(&original, ImageFormat::Png);
    let decoded_png = image::load_from_memory_with_format(&png_bytes, ImageFormat::Png)
        .expect("decode PNG from memory");
    let config = AsciiConverterConfig {
        target_width: Some(30),
        target_height: Some(15),
        color_mode: ColorMode::Monochrome,
        ramp: LuminanceRamp::Standard,
        invert: false,
    };
    let png_ascii = convert_image_to_ascii(&decoded_png, &config);
    assert!(!png_ascii.is_empty());
    let png_lines: Vec<&str> = png_ascii.lines().collect();
    assert_eq!(png_lines.len(), 15);

    // 2. Test BMP decode from memory
    let bmp_bytes = encode_image(&original, ImageFormat::Bmp);
    let decoded_bmp = image::load_from_memory_with_format(&bmp_bytes, ImageFormat::Bmp)
        .expect("decode BMP from memory");
    let bmp_ascii = convert_image_to_ascii(&decoded_bmp, &config);
    assert!(!bmp_ascii.is_empty());
    let bmp_lines: Vec<&str> = bmp_ascii.lines().collect();
    assert_eq!(bmp_lines.len(), 15);
}

#[test]
fn test_truecolor_ansi_emission_vs_plaintext() {
    let img = generate_quadrant_image(40, 40);

    // Monochrome mode: must have ZERO escape sequences
    let mono_config = AsciiConverterConfig {
        target_width: Some(20),
        target_height: Some(10),
        color_mode: ColorMode::Monochrome,
        ramp: LuminanceRamp::Standard,
        invert: false,
    };
    let mono_art = convert_image_to_ascii(&img, &mono_config);
    assert!(
        !mono_art.contains("\x1b["),
        "Monochrome ASCII art must contain zero ANSI escape codes"
    );

    // TrueColor mode: must contain 24-bit ANSI sequences (\x1b[38;2;r;g;bm)
    let tc_config = AsciiConverterConfig {
        color_mode: ColorMode::TrueColor,
        ..mono_config.clone()
    };
    let tc_art = convert_image_to_ascii(&img, &tc_config);
    assert!(
        tc_art.contains("\x1b[38;2;"),
        "TrueColor ASCII art must contain 24-bit RGB ANSI escape sequences"
    );
    assert!(
        tc_art.contains("\x1b[0m"),
        "TrueColor ASCII art must include ANSI color resets"
    );

    // Ansi256 mode: must contain 256-color ANSI sequences (\x1b[38;5;Nm)
    let ansi256_config = AsciiConverterConfig {
        color_mode: ColorMode::Ansi256,
        ..mono_config.clone()
    };
    let ansi256_art = convert_image_to_ascii(&img, &ansi256_config);
    assert!(
        ansi256_art.contains("\x1b[38;5;"),
        "Ansi256 ASCII art must contain 256-color ANSI codes"
    );

    // Grayscale mode: must contain grayscale 256-color ANSI sequences
    let gray_config = AsciiConverterConfig {
        color_mode: ColorMode::Grayscale,
        ..mono_config
    };
    let gray_art = convert_image_to_ascii(&img, &gray_config);
    assert!(
        gray_art.contains("\x1b[38;5;"),
        "Grayscale ASCII art must contain ANSI codes"
    );
}

#[test]
fn test_luminance_ramps() {
    let img = generate_quadrant_image(40, 40);

    // Standard ramp
    let std_cfg = AsciiConverterConfig {
        target_width: Some(20),
        target_height: Some(10),
        color_mode: ColorMode::Monochrome,
        ramp: LuminanceRamp::Standard,
        invert: false,
    };
    let std_art = convert_image_to_ascii(&img, &std_cfg);
    assert!(std_art.contains('@') || std_art.contains('%') || std_art.contains('#'));

    // Detailed ramp
    let detailed_cfg = AsciiConverterConfig {
        ramp: LuminanceRamp::Detailed,
        ..std_cfg.clone()
    };
    let detailed_art = convert_image_to_ascii(&img, &detailed_cfg);
    assert!(!detailed_art.is_empty());

    // Unicode block ramp
    let blocks_cfg = AsciiConverterConfig {
        ramp: LuminanceRamp::Blocks,
        ..std_cfg
    };
    let blocks_art = convert_image_to_ascii(&img, &blocks_cfg);
    assert!(
        blocks_art.contains('█') || blocks_art.contains('▓') || blocks_art.contains('▒'),
        "Blocks ramp must produce Unicode shade/block characters"
    );
}

#[test]
fn test_cow_format_wrapper_with_thoughts_and_eyes() {
    let img = generate_quadrant_image(50, 50);

    // Convert in-memory image to standard .cow format
    let cow_template = image_to_cow_from_image(&img, "quad_mascot", Some(30))
        .expect("image_to_cow_from_image should succeed");

    // 1. Verify Perl heredoc structure
    assert!(cow_template.starts_with("$the_cow = <<\"EOC\";"));
    assert!(cow_template.contains("EOC"));

    // 2. Verify $thoughts and $eyes placeholders
    assert!(
        cow_template.contains("$thoughts"),
        "Generated .cow must contain $thoughts speech bubble anchor"
    );
    assert!(
        cow_template.contains("$eyes"),
        "Generated .cow must contain $eyes facial anchor"
    );

    // 3. Test cow::expand_cow expansion with speech bubble pointer
    let expanded_speech = cow::expand_cow(&cow_template, "oo", "U", "\\");
    assert!(!expanded_speech.contains("$eyes"), "$eyes must be expanded");
    assert!(
        !expanded_speech.contains("$thoughts"),
        "$thoughts must be expanded"
    );
    assert!(
        expanded_speech.contains("oo"),
        "Custom eyes must appear in expanded cow"
    );
    assert!(
        expanded_speech.contains('\\'),
        "Speech bubble pointer must appear in expanded cow"
    );

    // 4. Test cow::expand_cow expansion with thought bubble pointer
    let expanded_thought = cow::expand_cow(&cow_template, "$$", "U", "o");
    assert!(!expanded_thought.contains("$eyes"));
    assert!(!expanded_thought.contains("$thoughts"));
    assert!(expanded_thought.contains("$$"), "Custom eyes must appear");
    assert!(
        expanded_thought.contains('o'),
        "Thought bubble pointer must appear"
    );

    // 5. Test speech bubble scene composition
    let scene_speech =
        cow::compose_scene_with_mode(&expanded_speech, "Hello from image cow!", false);
    assert!(scene_speech.contains("Hello from image cow!"));
    assert!(scene_speech.contains("oo"));
    assert!(scene_speech.contains('\\'));

    // 6. Test thought bubble scene composition
    let scene_thought = cow::compose_scene_with_mode(&expanded_thought, "Deep thoughts...", true);
    assert!(scene_thought.contains("Deep thoughts..."));
    assert!(scene_thought.contains("$$"));
}

#[test]
fn test_save_custom_cow_to_disk() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    std::fs::write(&config_file, "{}").unwrap();
    let original_config = std::env::var("FORGUM_CONFIG").ok();
    // Point config to temp directory
    std::env::set_var("FORGUM_CONFIG", &config_file);

    let cow_content = "$the_cow = <<\"EOC\";\n  $thoughts\n   $thoughts\n  ($eyes)\nEOC\n";
    let saved_path =
        save_custom_cow("test_mascot", cow_content).expect("save_custom_cow should succeed");

    assert!(saved_path.is_file());
    assert_eq!(
        saved_path.file_name().unwrap().to_str().unwrap(),
        "test_mascot.cow"
    );

    let read_back = std::fs::read_to_string(&saved_path).unwrap();
    assert_eq!(read_back, cow_content);

    // Also test image file conversion from disk
    let img_path = temp_dir.path().join("sample.png");
    let sample_img = generate_quadrant_image(40, 40);
    sample_img.save(&img_path).expect("save sample image");

    let ascii_from_disk = convert_image_path_to_ascii(&img_path, &AsciiConverterConfig::default())
        .expect("convert_image_path_to_ascii should succeed");
    assert!(!ascii_from_disk.is_empty());

    let cow_from_disk = image_to_cow(&img_path, "sample_cow", Some(30))
        .expect("image_to_cow from disk should succeed");
    assert!(cow_from_disk.contains("$the_cow"));

    // Restore environment
    if let Some(orig) = original_config {
        std::env::set_var("FORGUM_CONFIG", orig);
    } else {
        std::env::remove_var("FORGUM_CONFIG");
    }
}

#[test]
fn test_cli_image_subcommand_parsing() {
    // forgum image test.png --width 50 --height 25 --color ansi256 --ramp blocks --invert
    let argv = vec![
        "forgum".to_string(),
        "image".to_string(),
        "test.png".to_string(),
        "--width".to_string(),
        "50".to_string(),
        "--height".to_string(),
        "25".to_string(),
        "--color".to_string(),
        "ansi256".to_string(),
        "--ramp".to_string(),
        "blocks".to_string(),
        "--invert".to_string(),
    ];
    let (args, cmd) = parse_args(argv).expect("parse image subcommand");
    assert_eq!(args.command, Command::Image);

    if let Some(Commands::Image {
        path,
        width,
        height,
        color,
        ramp,
        save_cow,
        output,
        invert,
    }) = cmd
    {
        assert_eq!(path.to_str().unwrap(), "test.png");
        assert_eq!(width, Some(50));
        assert_eq!(height, Some(25));
        assert_eq!(color, "ansi256");
        assert_eq!(ramp, "blocks");
        assert_eq!(save_cow, None);
        assert_eq!(output, None);
        assert!(invert);
    } else {
        panic!("expected Commands::Image variant");
    }
}

#[test]
fn test_cli_image_subcommand_save_cow_option() {
    let argv = vec![
        "forgum".to_string(),
        "image".to_string(),
        "logo.png".to_string(),
        "--save-cow".to_string(),
        "my_logo".to_string(),
        "--output".to_string(),
        "custom.cow".to_string(),
    ];
    let (args, cmd) = parse_args(argv).expect("parse image save-cow");
    assert_eq!(args.command, Command::Image);

    if let Some(Commands::Image {
        path,
        save_cow,
        output,
        ..
    }) = cmd
    {
        assert_eq!(path.to_str().unwrap(), "logo.png");
        assert_eq!(save_cow, Some("my_logo".to_string()));
        assert_eq!(output.unwrap().to_str().unwrap(), "custom.cow");
    } else {
        panic!("expected Commands::Image variant");
    }
}

#[test]
fn test_cli_render_and_say_image_flag() {
    let _lock = ENV_MUTEX.lock().unwrap();
    // 1. forgum render --image mascot.png
    let argv_render = vec![
        "forgum".to_string(),
        "render".to_string(),
        "--image".to_string(),
        "mascot.png".to_string(),
    ];
    let (args_render, _) = parse_args(argv_render).expect("parse render --image");
    assert_eq!(
        args_render.image.as_ref().map(|p| p.to_str().unwrap()),
        Some("mascot.png")
    );

    // Build scene config and verify cfg.image is set
    let cfg = forgum_engine::cli::build_scene_config(&args_render).expect("build scene config");
    assert_eq!(cfg.image, Some("mascot.png".to_string()));

    // 2. forgum say --image logo.png echo "hello"
    let argv_say = vec![
        "forgum".to_string(),
        "say".to_string(),
        "--image".to_string(),
        "logo.png".to_string(),
        "echo".to_string(),
        "hello".to_string(),
    ];
    let (args_say, cmd_say) = parse_args(argv_say).expect("parse say --image");
    assert_eq!(
        args_say.image.as_ref().map(|p| p.to_str().unwrap()),
        Some("logo.png")
    );
    assert!(matches!(cmd_say, Some(Commands::Say { ref cmd }) if cmd == &["echo", "hello"]));
}
