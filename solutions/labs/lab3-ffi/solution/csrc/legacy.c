#include "legacy.h"

#include <stdlib.h>
#include <string.h>

/* Singly linked list of key/value entries. Simple on purpose -- this file
 * is "legacy" scaffolding for the FFI lab, not itself the point of the
 * lesson. */
typedef struct legacy_entry {
    char *key;
    char *value;
    struct legacy_entry *next;
} legacy_entry;

struct legacy_db {
    char *path;
    legacy_entry *head;
};

static char *dup_string(const char *s) {
    size_t len = strlen(s) + 1;
    char *copy = malloc(len);
    if (copy != NULL) {
        memcpy(copy, s, len);
    }
    return copy;
}

static legacy_entry *find_entry(legacy_db *db, const char *k) {
    for (legacy_entry *e = db->head; e != NULL; e = e->next) {
        if (strcmp(e->key, k) == 0) {
            return e;
        }
    }
    return NULL;
}

legacy_db *legacy_open(const char *path) {
    if (path == NULL) {
        return NULL;
    }

    legacy_db *db = malloc(sizeof(legacy_db));
    if (db == NULL) {
        return NULL;
    }

    db->path = dup_string(path);
    if (db->path == NULL) {
        free(db);
        return NULL;
    }

    db->head = NULL;
    return db;
}

int legacy_put(legacy_db *db, const char *k, const char *v) {
    if (db == NULL || k == NULL || v == NULL) {
        return -1;
    }

    legacy_entry *existing = find_entry(db, k);
    if (existing != NULL) {
        char *new_value = dup_string(v);
        if (new_value == NULL) {
            return -1;
        }
        free(existing->value);
        existing->value = new_value;
        return 0;
    }

    legacy_entry *entry = malloc(sizeof(legacy_entry));
    if (entry == NULL) {
        return -1;
    }

    entry->key = dup_string(k);
    entry->value = dup_string(v);
    if (entry->key == NULL || entry->value == NULL) {
        free(entry->key);
        free(entry->value);
        free(entry);
        return -1;
    }

    entry->next = db->head;
    db->head = entry;
    return 0;
}

const char *legacy_get(legacy_db *db, const char *k) {
    if (db == NULL || k == NULL) {
        return NULL;
    }

    legacy_entry *entry = find_entry(db, k);
    return entry == NULL ? NULL : entry->value;
}

void legacy_close(legacy_db *db) {
    if (db == NULL) {
        return;
    }

    legacy_entry *e = db->head;
    while (e != NULL) {
        legacy_entry *next = e->next;
        free(e->key);
        free(e->value);
        free(e);
        e = next;
    }

    free(db->path);
    free(db);
}
