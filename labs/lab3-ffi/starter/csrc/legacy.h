#ifndef LEGACY_H
#define LEGACY_H

#ifdef __cplusplus
extern "C" {
#endif

/*
 * legacy.h -- mock "legacy" C key/value store.
 *
 * This header describes a small, deliberately old-school C API: an opaque
 * handle malloc'd on open and only ever freed by legacy_close, an insert
 * operation that copies its string arguments in, and a lookup operation
 * that hands back a pointer the caller must NOT free. It exists purely so
 * the FFI lab has a real (if trivial) foreign library to wrap safely from
 * Rust. Attendees never edit this file or legacy.c.
 */

typedef struct legacy_db legacy_db;

/* Allocate and initialize a new in-memory database handle.
 *
 * Returns NULL if `path` is NULL or allocation fails. The returned handle
 * must eventually be passed to legacy_close exactly once. */
legacy_db *legacy_open(const char *path);

/* Insert or overwrite the value stored under `k`.
 *
 * Returns 0 on success, non-zero on failure (NULL arguments or allocation
 * failure). Both `k` and `v` must be NUL-terminated; they are copied
 * internally, so the caller retains ownership of the pointers passed in
 * and may free/reuse them immediately after the call returns. */
int legacy_put(legacy_db *db, const char *k, const char *v);

/* Look up the value stored under `k`.
 *
 * Returns NULL if `db`/`k` is NULL or the key is not present. On success,
 * returns a pointer OWNED BY THE DATABASE: it stays valid until the next
 * legacy_put call for the same key, or until legacy_close. The caller
 * must NOT free this pointer. */
const char *legacy_get(legacy_db *db, const char *k);

/* Release every resource owned by `db`, including all stored keys/values
 * and the handle itself. `db` may be NULL, in which case this is a no-op.
 * After this call the pointer is invalid and must never be used again
 * (including being passed to legacy_close a second time). */
void legacy_close(legacy_db *db);

#ifdef __cplusplus
}
#endif

#endif /* LEGACY_H */
