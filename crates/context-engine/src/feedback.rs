use super::*;

const MAX_FEEDBACK_ITEMS: usize = 20;
const MAX_FEEDBACK_REASON_CHARS: usize = 240;

pub fn context_feedback_write(
    root: &Path,
    generated_at_epoch_ms: u128,
    rating: ContextFeedbackRating,
    reason: &str,
) -> io::Result<ContextFeedbackWriteResult> {
    let reason = bounded_reason(reason)?;
    let mut stored = load_context_feedback(root)?;
    let recorded_at_epoch_ms = now_epoch_ms()?;
    let feedback = ContextFeedback {
        id: format!("{generated_at_epoch_ms}-{recorded_at_epoch_ms}"),
        generated_at_epoch_ms,
        recorded_at_epoch_ms,
        rating,
        reason,
    };

    stored.feedback.push(feedback.clone());
    if stored.feedback.len() > MAX_FEEDBACK_ITEMS {
        let overflow = stored.feedback.len() - MAX_FEEDBACK_ITEMS;
        stored.feedback.drain(0..overflow);
    }
    persist_context_feedback(root, &stored)?;

    Ok(ContextFeedbackWriteResult { feedback })
}

pub fn context_feedback_list(root: &Path, limit: usize) -> io::Result<ContextFeedbackListResult> {
    let capped_limit = limit.clamp(1, MAX_FEEDBACK_ITEMS);
    let stored = load_context_feedback(root)?;
    let total = stored.feedback.len();
    let feedback = stored
        .feedback
        .into_iter()
        .rev()
        .take(capped_limit)
        .collect::<Vec<_>>();

    Ok(ContextFeedbackListResult {
        feedback,
        omitted_count: total.saturating_sub(capped_limit),
    })
}

fn bounded_reason(reason: &str) -> io::Result<String> {
    let trimmed = reason.trim();
    if trimmed.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "context feedback reason cannot be empty",
        ));
    }

    Ok(trimmed.chars().take(MAX_FEEDBACK_REASON_CHARS).collect())
}
