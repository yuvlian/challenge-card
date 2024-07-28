use std::fs::File;
use std::io::Write;
use std::path::Path;

use image::{Rgba, RgbaImage};
use reqwest::Client;
use rusttype::{point, Font, PositionedGlyph, Scale};

mod get_player;
use get_player::UserData;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ask user for uid n stuff
    let mut input = String::new();
    println!("Enter UID, MOC STAR, PF STAR, AS STAR\nExample: 80000000,36,12,12\n");
    // challenge data can be private so yeah
    // no im not adding a check whether it is or not, its only 6 digits, screw off

    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();
    let parts: Vec<&str> = input.split(',').collect();

    if parts.len() < 4 {
        println!("Error: Not enough input values provided.");
        todo!() // make the program panic!
    }

    // uid
    let first_number: u32 = match parts[0].parse() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Error: Invalid number format for the first value.");
            return Ok(());
        }
    };

    let second_number = parts[1]; // MoC star
    let third_number = parts[2]; // PF
    let fourth_number = parts[3]; // AS
    let uid = first_number;

    let player: UserData = get_player::user(uid).await?;

    // name, signature, level, world level
    let pn = player.name;
    let ps = player.signature;
    let lv = format!("LV {}", player.lv);
    let eq = format!("EQ {}", player.eq);

    // url for avatar icon
    let av_url = format!(
        r"https://raw.githubusercontent.com/Mar-7th/StarRailRes/master/{}",
        player.av
    );
    println!("\nAccount icon image url: {}", av_url);
    // path for avatar icon temp file
    let av_path = "temp_avatar.png";

    // Download the avatar icon
    download_image(&av_url, av_path).await?;

    // canvas image
    let mut img: RgbaImage =
        image::open(r"assets\template.png")?.to_rgba8();

    // bold font
    let bold_font_data = include_bytes!(r"assets\GlacialIndifference-Bold.otf") as &[u8];
    let bold_font = Font::try_from_bytes(bold_font_data).unwrap();

    // regular font
    let regular_font_data = include_bytes!(r"assets\GlacialIndifference-Regular.otf") as &[u8];
    let regular_font = Font::try_from_bytes(regular_font_data).unwrap();

    let image_overlays = vec![
        (av_path, 60, 60, 368, 368), // image, x, y, width, height
    ];

    // loop through available images
    for (path, x, y, width, height) in image_overlays {
        match add_image_overlay(&mut img, path, x, y, width, height) {
            Ok(_) => {}
            Err(e) => eprintln!("Error adding overlay {}: {}", path, e),
        }
    }

    let text_overlays = vec![
        (second_number, 317.0, 929.0, 58.0, &bold_font), // moc star
        (third_number, 937.0, 929.0, 58.0, &bold_font),  // pf star
        (fourth_number, 1552.0, 929.0, 58.0, &bold_font), // AS star
        (&lv, 475.0, 226.0, 56.0, &bold_font),           // account lvl
        (&eq, 656.0, 226.0, 56.0, &bold_font),           // world lvl
        (&pn, 457.0, 140.0, 101.0, &regular_font),       // player name
        (&ps, 459.0, 306.0, 58.0, &regular_font),        // player signature
    ];

    // loop through available texts
    for (text, x, y, scale_x, font) in text_overlays {
        add_text_overlay(&mut img, font, text, x, y, scale_x);
    }

    // save img
    img.save("output.png")?;
    println!("Showcase card image saved successfully");

    // delete temp file
    std::fs::remove_file(av_path)?;
    println!("Deleted temp file successfully");
    Ok(())
}

// DO NOT SCROLL ANY FURTHER!
// MESSY CODE!!
// YOU HAVE BEEN WARNED!!

fn add_image_overlay(
    base_image: &mut RgbaImage,
    overlay_path: &str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to open overlay image: {}", overlay_path);

    if !Path::new(overlay_path).exists() {
        return Err(format!("Overlay image not found: {}", overlay_path).into());
    }

    let overlay = image::open(overlay_path)?;
    let overlay_resized = overlay.resize(width, height, image::imageops::FilterType::Nearest);
    let overlay_rgba = overlay_resized.to_rgba8();
    println!(
        "Overlay image opened and resized successfully. New dimensions: {}x{}",
        overlay_resized.width(),
        overlay_resized.height()
    );

    let (base_width, base_height) = base_image.dimensions();

    for (i, j, pixel) in overlay_rgba.enumerate_pixels() {
        let x_offset = x as i32 + i as i32;
        let y_offset = y as i32 + j as i32;

        if x_offset >= 0
            && x_offset < base_width as i32
            && y_offset >= 0
            && y_offset < base_height as i32
        {
            let x = x_offset as u32;
            let y = y_offset as u32;
            let base_pixel = base_image.get_pixel_mut(x, y);
            *base_pixel = blend_image_pixel(*base_pixel, *pixel);
        }
    }

    println!("Overlay applied successfully");
    Ok(())
}

fn blend_image_pixel(bottom: Rgba<u8>, top: Rgba<u8>) -> Rgba<u8> {
    let alpha_top = top[3] as f32 / 255.0;
    let alpha_bottom = bottom[3] as f32 / 255.0;
    let alpha_out = alpha_top + alpha_bottom * (1.0 - alpha_top);

    if alpha_out == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }

    let red =
        ((top[0] as f32 * alpha_top + bottom[0] as f32 * (1.0 - alpha_top)) / alpha_out) as u8;
    let green =
        ((top[1] as f32 * alpha_top + bottom[1] as f32 * (1.0 - alpha_top)) / alpha_out) as u8;
    let blue =
        ((top[2] as f32 * alpha_top + bottom[2] as f32 * (1.0 - alpha_top)) / alpha_out) as u8;
    let alpha = (alpha_out * 255.0) as u8;

    Rgba([red, green, blue, alpha])
}

fn add_text_overlay(img: &mut RgbaImage, font: &Font, text: &str, x: f32, y: f32, scale_x: f32) {
    let scale = Scale {
        x: scale_x,
        y: scale_x,
    };
    let color = Rgba([255, 255, 255, 255]);
    let offset = point(x, y);
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, offset).collect();
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let x = x + bounding_box.min.x as u32;
                let y = y + bounding_box.min.y as u32;
                if x < img.width() && y < img.height() {
                    let pixel = img.get_pixel_mut(x, y);
                    let alpha = (v * 255.0) as u8;
                    *pixel = blend_text_pixel(*pixel, Rgba([color[0], color[1], color[2], alpha]));
                }
            });
        }
    }
}

fn blend_text_pixel(bottom: Rgba<u8>, top: Rgba<u8>) -> Rgba<u8> {
    let alpha = top[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    Rgba([
        (top[0] as f32 * alpha + bottom[0] as f32 * inv_alpha) as u8,
        (top[1] as f32 * alpha + bottom[1] as f32 * inv_alpha) as u8,
        (top[2] as f32 * alpha + bottom[2] as f32 * inv_alpha) as u8,
        255,
    ])
}

async fn download_image(url: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let mut file = File::create(save_path)?;
        let content = response.bytes().await?;
        std::io::copy(&mut content.as_ref(), &mut file)?;
        Ok(())
    } else {
        Err(format!("Failed to download image: {}", response.status()).into())
    }
}
