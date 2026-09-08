//! Regression guard for the channel-image-cache deployment (task 3.4).
//!
//! The cover-image cache must live inside the already-mounted `db` volume
//! (`/app/db/images` in the container) so it survives container recreation
//! without any new Docker volume entry. This test locks `docker-compose.yml`
//! and `images_dir()` to that contract:
//!   1. `docker-compose.yml` never gains a service mount or named volume for
//!      the image cache (only the pre-existing `audios` and `db` volumes).
//!   2. `images_dir()` resolves under the `/app/db` mount in the container and
//!      never into the audio volume (a channel slug such as `images` must not
//!      collide with cache files).

use std::path::Path;

fn read_repo_file(name: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
        .unwrap_or_else(|e| panic!("reading {name}: {e}"))
}

#[test]
fn compose_has_no_new_volume_for_image_cache() {
    let compose = read_repo_file("docker-compose.yml");

    // The pre-existing service mounts remain the only ones.
    assert!(
        compose.contains("audios:/app/audios"),
        "audios service mount must stay"
    );
    assert!(
        compose.contains("db:/app/db"),
        "the cache relies on the existing `db` volume mounted at /app/db"
    );

    // No mount or named volume may reference the image cache (3.4: "no new
    // volume entries").
    assert!(
        !compose.contains("images:/app/"),
        "image cache must not add a service mount"
    );
    assert!(
        !compose.contains("images:"),
        "image cache must not add a named volume"
    );

    // The named-volumes block still lists exactly the two original volumes.
    let named = compose.split("volumes:").last().unwrap_or_default();
    assert!(
        named.contains("audios: {}"),
        "named `audios` volume must remain"
    );
    assert!(named.contains("db: {}"), "named `db` volume must remain");
    assert!(
        !named.contains("images"),
        "no named volume may be added for the image cache"
    );
}

#[test]
fn images_dir_maps_into_the_db_volume_only() {
    let config_src = read_repo_file("src/models/config.rs");

    // The container branch of `images_dir()` must resolve under the existing
    // `db` mount so cached images persist across container recreation.
    assert!(
        config_src.contains("/app/db/images"),
        "images_dir() must resolve under the `db` mount in the container"
    );
    // It must never map into the audio volume: a channel slug `images` would
    // collide with a top-level cache directory there.
    assert!(
        !config_src.contains("/app/audios/images"),
        "the image cache must not live in the audio volume"
    );
}
