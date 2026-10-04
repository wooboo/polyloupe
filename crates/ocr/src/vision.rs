//! Apple Vision text recognition (`VNRecognizeTextRequest`).

use anyhow::{Context as _, anyhow};
use objc2::AnyThread as _;
use objc2::rc::Retained;
use objc2_core_foundation::CFData;
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo,
};
use objc2_foundation::{NSArray, NSDictionary, NSString};
use objc2_vision::{
    VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
};

use crate::{Rect, TextLine};

/// BCP 47 tags passed to Vision as a hint, in priority order.
const LANGUAGES: [&str; 6] = ["pl-PL", "en-US", "da-DK", "sv-SE", "fi-FI", "nb-NO"];

pub fn recognize(image: &image::RgbaImage) -> anyhow::Result<Vec<TextLine>> {
    let (width, height) = image.dimensions();
    let cg_image = to_cg_image(image).context("creating CGImage")?;

    let request = VNRecognizeTextRequest::new();
    request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
    request.setUsesLanguageCorrection(true);
    request.setAutomaticallyDetectsLanguage(true);
    let languages: Vec<Retained<NSString>> =
        LANGUAGES.iter().map(|l| NSString::from_str(l)).collect();
    request.setRecognitionLanguages(&NSArray::from_retained_slice(&languages));

    let handler = unsafe {
        VNImageRequestHandler::initWithCGImage_options(
            VNImageRequestHandler::alloc(),
            &cg_image,
            &NSDictionary::new(),
        )
    };
    let requests: Retained<NSArray<VNRequest>> =
        NSArray::from_retained_slice(&[Retained::into_super(Retained::into_super(
            request.clone(),
        ))]);
    handler.performRequests_error(&requests).map_err(|e| {
        anyhow!(
            "Vision text recognition failed: {}",
            e.localizedDescription()
        )
    })?;

    let (w, h) = (width as f32, height as f32);
    let mut lines = Vec::new();
    for observation in request.results().unwrap_or_default().iter() {
        let Some(candidate) = observation.topCandidates(1).firstObject() else {
            continue;
        };
        // Vision boxes are normalized with the origin at the bottom-left.
        let b = unsafe { observation.boundingBox() };
        lines.push(TextLine {
            text: candidate.string().to_string(),
            bounds: Rect {
                x: b.origin.x as f32 * w,
                y: (1.0 - (b.origin.y + b.size.height) as f32) * h,
                width: b.size.width as f32 * w,
                height: b.size.height as f32 * h,
            },
            confidence: candidate.confidence(),
        });
    }
    Ok(lines)
}

fn to_cg_image(image: &image::RgbaImage) -> Option<objc2_core_foundation::CFRetained<CGImage>> {
    let (width, height) = image.dimensions();
    let data = CFData::from_bytes(image.as_raw());
    let provider = CGDataProvider::with_cf_data(Some(&data))?;
    let color_space = CGColorSpace::new_device_rgb()?;
    unsafe {
        CGImage::new(
            width as usize,
            height as usize,
            8,
            32,
            width as usize * 4,
            Some(&color_space),
            CGBitmapInfo(CGImageAlphaInfo::PremultipliedLast.0),
            Some(&provider),
            std::ptr::null(),
            false,
            CGColorRenderingIntent::RenderingIntentDefault,
        )
    }
}
