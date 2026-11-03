//! Compatibility boundary frozen before the Pagic rename. Never rename the PDF markers.
use recto_engine::{Changes, Edit, EditError, MarkKind, Protection, Session};
use recto_render::{PageRenderer, RenderConfig, RenderRequest, RequestKind};
use std::sync::Arc;

const PDF: &[u8] = include_bytes!("fixtures/recto-marks.pdf");
fn open(session: &mut Session, bytes: Arc<Vec<u8>>, password: Option<&str>) -> recto_engine::DocId {
    session.open("legacy.pdf", None, bytes, password).unwrap()
}
fn text(session: &Session, id: recto_engine::DocId) -> String {
    let doc = session.get(id).unwrap();
    let mut renderer = PageRenderer::new(doc.bytes.clone(), RenderConfig { password: doc.password.as_deref().map(Arc::from), ..Default::default() });
    renderer.render(RenderRequest { page: 0, kind: RequestKind::Text, ..Default::default() }).text.unwrap().plain_text()
}

#[test]
fn historical_wrappers_and_private_marks_survive_edit_save_reopen_and_removal() {
    assert!(PDF.windows(b"q %Recto\n".len()).any(|w| w == b"q %Recto\n"));
    assert!(PDF.windows(b"/PCMark /Wrap".len()).any(|w| w == b"/PCMark /Wrap"));
    for full in [false, true] {
        let mut session = Session::new();
        let id = open(&mut session, Arc::new(PDF.to_vec()), None);
        assert_eq!(session.get(id).unwrap().info_value("Producer").as_deref(), Some("Recto"));
        assert!(session.get(id).unwrap().marks().contains(&MarkKind::Watermark));
        assert!(text(&session, id).contains("LEGACY WATERMARK"));
        session.apply(id, Edit::SetInfo { key: "Title".into(), value: "Edited in the independent application".into() }).unwrap();
        let saved = if full { session.save_full_bytes(id) } else { session.save_bytes(id) }.unwrap();
        let mut reopened = Session::new();
        let id = open(&mut reopened, saved, None);
        assert!(text(&reopened, id).contains("Original document text"));
        assert!(reopened.get(id).unwrap().marks().contains(&MarkKind::Watermark));
        reopened.apply(id, Edit::RemoveMarks { kind: MarkKind::Watermark }).unwrap();
        assert!(!text(&reopened, id).contains("LEGACY WATERMARK"));
        let mut again = Session::new();
        let id = open(&mut again, reopened.save_bytes(id).unwrap(), None);
        assert!(text(&again, id).contains("Original document text"));
        assert!(again.get(id).unwrap().marks().is_empty());
    }
}

#[test]
fn legacy_pdf_encryption_and_owner_permissions_remain_enforced() {
    let mut session = Session::new();
    let id = open(&mut session, Arc::new(PDF.to_vec()), None);
    session
        .apply(
            id,
            Edit::Protect(Protection {
                open_password: Some("reader".into()),
                permissions_password: Some("owner".into()),
                changes: Changes::None,
                copy: false,
                ..Default::default()
            }),
        )
        .unwrap();
    let saved = session.save_bytes(id).unwrap();
    let mut reader = Session::new();
    assert!(reader.open("legacy.pdf", None, saved.clone(), None).is_err());
    let id = open(&mut reader, saved.clone(), Some("reader"));
    assert!(reader.get(id).unwrap().info.encrypted);
    assert!(matches!(reader.apply(id, Edit::RotatePages { pages: vec![0], degrees: 90 }), Err(EditError::NotPermitted(_))));
    assert!(matches!(reader.apply(id, Edit::RemoveProtection), Err(EditError::NotPermitted(_))));
    let mut owner = Session::new();
    let id = open(&mut owner, saved, Some("owner"));
    owner.apply(id, Edit::RotatePages { pages: vec![0], degrees: 90 }).unwrap();
    let saved = owner.save_bytes(id).unwrap();
    assert!(recto_cos::Document::open(saved.clone()).is_err());
    let id = open(&mut Session::new(), saved.clone(), Some("reader"));
    assert_eq!(id, recto_engine::DocId(1));
    assert!(recto_cos::Document::open_with_password(saved, Some("reader")).is_ok());
}

#[test]
fn existing_signed_fixture_is_not_rewritten_or_sanitized() {
    let signed = Arc::new(include_bytes!("../../../sign/tests/data/openssl-signed.pdf").to_vec());
    let mut session = Session::new();
    let id = open(&mut session, signed.clone(), None);
    assert!(session.get(id).unwrap().is_signed());
    assert_eq!(*session.save_bytes(id).unwrap(), *signed);
    assert_eq!(session.save_full_bytes(id).unwrap_err(), EditError::Signed);
    assert!(matches!(session.apply(id, Edit::Sanitize), Err(EditError::SignedRewrite(_))));
    assert_eq!(*session.save_bytes(id).unwrap(), *signed);
}
