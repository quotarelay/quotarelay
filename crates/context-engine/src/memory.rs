use super::*;

pub fn memory_write(
    root: &Path,
    title: &str,
    content: &str,
    tags: &[String],
) -> io::Result<MemoryWriteResult> {
    let now = now_epoch_ms()?;
    let mut stored = load_memory_notes(root)?;
    let note = MemoryNote {
        id: format!("mem-{}", unique_epoch_nanos()?),
        title: title.to_string(),
        content: content.to_string(),
        tags: tags.to_vec(),
        created_at_epoch_ms: now,
        updated_at_epoch_ms: now,
    };
    stored.notes.push(note.clone());
    persist_memory_notes(root, &stored)?;
    Ok(MemoryWriteResult { note })
}

pub fn memory_update(
    root: &Path,
    id: &str,
    title: Option<&str>,
    content: Option<&str>,
    tags: Option<&[String]>,
) -> io::Result<MemoryUpdateResult> {
    let mut stored = load_memory_notes(root)?;
    let note = stored
        .notes
        .iter_mut()
        .find(|note| note.id == id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("memory note {id} not found"),
            )
        })?;

    let mut changed = false;

    if let Some(title) = title {
        note.title = title.to_string();
        changed = true;
    }
    if let Some(content) = content {
        note.content = content.to_string();
        changed = true;
    }
    if let Some(tags) = tags {
        note.tags = tags.to_vec();
        changed = true;
    }

    if !changed {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "memory update requires at least one field to change",
        ));
    }

    note.updated_at_epoch_ms = now_epoch_ms()?;
    let updated = note.clone();
    persist_memory_notes(root, &stored)?;

    Ok(MemoryUpdateResult { note: updated })
}

pub fn memory_delete(root: &Path, id: &str) -> io::Result<MemoryDeleteResult> {
    let mut stored = load_memory_notes(root)?;
    let removed = stored
        .notes
        .iter()
        .position(|note| note.id == id)
        .map(|index| stored.notes.remove(index));

    if removed.is_some() {
        persist_memory_notes(root, &stored)?;
    }

    Ok(MemoryDeleteResult { note: removed })
}

pub fn memory_export(root: &Path, limit: usize) -> io::Result<MemoryExportResult> {
    let stored = load_memory_notes(root)?;
    let capped_limit = limit.clamp(1, MAX_MEMORY_TRANSFER_NOTES);
    let total_count = stored.notes.len();
    let notes = stored
        .notes
        .into_iter()
        .take(capped_limit)
        .collect::<Vec<_>>();

    Ok(MemoryExportResult {
        payload: MemoryExportPayload { notes },
        omitted_count: total_count.saturating_sub(capped_limit),
    })
}

pub fn memory_import(root: &Path, payload: MemoryExportPayload) -> io::Result<MemoryImportResult> {
    let mut stored = load_memory_notes(root)?;
    let incoming_count = payload.notes.len();
    let mut imported_count = 0;
    let mut replaced_count = 0;
    let mut omitted_count = incoming_count.saturating_sub(MAX_MEMORY_TRANSFER_NOTES);

    for note in payload.notes.into_iter().take(MAX_MEMORY_TRANSFER_NOTES) {
        if let Some(existing) = stored
            .notes
            .iter_mut()
            .find(|existing| existing.id == note.id)
        {
            *existing = note;
            replaced_count += 1;
        } else {
            stored.notes.push(note);
            imported_count += 1;
        }
    }

    if stored.notes.len() > MAX_MEMORY_TRANSFER_NOTES {
        omitted_count += stored.notes.len() - MAX_MEMORY_TRANSFER_NOTES;
        stored.notes.truncate(MAX_MEMORY_TRANSFER_NOTES);
    }

    persist_memory_notes(root, &stored)?;

    Ok(MemoryImportResult {
        imported_count,
        replaced_count,
        omitted_count,
    })
}

pub fn memory_read(root: &Path, id: &str) -> io::Result<Option<MemoryNote>> {
    let stored = load_memory_notes(root)?;
    Ok(stored.notes.into_iter().find(|note| note.id == id))
}

pub fn memory_search(root: &Path, query: &str, limit: usize) -> io::Result<MemorySearchResult> {
    let capped_limit = limit.clamp(1, MAX_CONTEXT_ITEMS);
    let mut matches = find_memory_notes(root, query)?;
    let omitted_count = matches.len().saturating_sub(capped_limit);
    matches.truncate(capped_limit);

    Ok(MemorySearchResult {
        notes: matches,
        omitted_count,
    })
}

pub(crate) fn find_memory_notes(root: &Path, query: &str) -> io::Result<Vec<MemoryNote>> {
    let normalized_query = query.to_ascii_lowercase();
    let stored = load_memory_notes(root)?;
    let mut matches = stored
        .notes
        .into_iter()
        .filter(|note| {
            note.title.to_ascii_lowercase().contains(&normalized_query)
                || note
                    .content
                    .to_ascii_lowercase()
                    .contains(&normalized_query)
                || note
                    .tags
                    .iter()
                    .any(|tag| tag.to_ascii_lowercase().contains(&normalized_query))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.updated_at_epoch_ms.cmp(&left.updated_at_epoch_ms));
    Ok(matches)
}

pub(crate) fn pack_memory_notes(
    query: &str,
    notes: Vec<MemoryNote>,
    limit: usize,
) -> (Vec<ContextMemoryNote>, Vec<OmissionReason>) {
    let total_notes = notes.len();
    let mut packed = Vec::new();
    let mut omissions = Vec::new();
    let mut remaining_bytes = MAX_MEMORY_PACK_BYTES;
    let mut byte_budget_hit = false;

    for note in notes.into_iter().take(limit) {
        let Some((content, truncated)) = fit_within_budget(&note.content, &mut remaining_bytes)
        else {
            byte_budget_hit = true;
            break;
        };
        packed.push(ContextMemoryNote {
            id: note.id,
            title: note.title,
            content,
            reason: InclusionReason {
                kind: InclusionReasonKind::MemoryNoteMatch,
                detail: format!("Durable memory matched query '{query}'."),
            },
        });
        if truncated {
            byte_budget_hit = true;
            break;
        }
    }

    if packed.is_empty() && total_notes == 0 {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::NoMemoryMatches,
            detail: format!("No durable memory matched query '{query}'."),
        });
    }
    if total_notes > limit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ItemLimitReached,
            detail: format!(
                "Additional memory notes omitted because item limit {limit} was reached."
            ),
        });
    }
    if byte_budget_hit {
        omissions.push(OmissionReason {
            kind: OmissionReasonKind::ByteBudgetReached,
            detail: format!("Packed memory notes reached byte budget {MAX_MEMORY_PACK_BYTES}."),
        });
    }

    (packed, omissions)
}
