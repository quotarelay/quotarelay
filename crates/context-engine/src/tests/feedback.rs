use crate::{
    context_feedback_list, context_feedback_write, inspect_local_state, ContextFeedbackRating,
};

use super::common::temp_repo;

#[test]
fn context_feedback_is_bounded_local_and_inspectable() {
    let root = temp_repo();

    for index in 0..25 {
        context_feedback_write(
            &root,
            index,
            ContextFeedbackRating::Useful,
            "focused context packet",
        )
        .expect("feedback should write");
    }

    let feedback = context_feedback_list(&root, 5).expect("feedback should list");
    assert_eq!(feedback.feedback.len(), 5);
    assert_eq!(feedback.omitted_count, 15);
    assert_eq!(feedback.feedback[0].generated_at_epoch_ms, 24);
    assert_eq!(feedback.feedback[0].rating, ContextFeedbackRating::Useful);

    let state = inspect_local_state(&root).expect("state should inspect");
    assert_eq!(state.context_feedback.present, true);
    assert_eq!(state.context_feedback.item_count, 20);
}

#[test]
fn context_feedback_rejects_empty_reasons() {
    let root = temp_repo();
    let error = context_feedback_write(&root, 1, ContextFeedbackRating::NotUseful, "   ")
        .expect_err("empty reason should fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
}
