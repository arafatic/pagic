//! Reproduce the contributor-original compatibility PDF with the baseline Recto writer.
//! Do not regenerate during tests: later writers must continue to read the frozen fixture.
use recto_engine::{Edit, Session, Watermark};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::new().with_clock(|| 1_700_000_000);
    let bytes = session.create_from_text("Legacy Recto document", "Original document text")?;
    let id = session.open("recto-marks.pdf", None, bytes, None)?;
    session.apply(
        id,
        Edit::AddWatermark {
            pages: vec![0],
            settings: Watermark { text: "LEGACY WATERMARK".into(), ..Default::default() },
            replace: false,
            file: None,
        },
    )?;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/it/fixtures/recto-marks.pdf");
    std::fs::write(path, &*session.save_bytes(id)?)?;
    Ok(())
}
