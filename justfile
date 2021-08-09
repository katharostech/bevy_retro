dev_features:=""

# List the justfile recipes
list:
    just --list

# Generate the README from the lib.rs docs
readme:
    cargo doc2readme --template README.j2 --out README.md

# Build Bevy Retrograde
build:
    cargo build --features {{dev_features}}

# Run an example
run-example example='hello_world':
    cargo run --example {{example}}

# Build an example for web and host it on a local webserver
run-example-web example='hello_world':
    cargo build --example {{example}} --features {{dev_features}} --target wasm32-unknown-unknown
    wasm-bindgen-0.2.72 --out-dir target/wasm/{{example}} --target web target/wasm32-unknown-unknown/debug/examples/{{example}}.wasm
    cp wasm_resources/index.tpl.html target/wasm/{{example}}/index.html
    sed -i s/\$example/{{example}}/ target/wasm/{{example}}/index.html
    ln -fs ../../../assets target/wasm/{{example}}
    basic-http-server target/wasm/{{example}}

# Build the documentation
doc *args:
    cargo doc --features {{dev_features}} {{args}}

# Publish all of the crates
publish confirm="":
    @if [ "{{confirm}}" = "yes I'm sure" ]; then \
        cd crates/bevy_retrograde_macros && cargo publish --no-verify && cd ../../ && \
        cd crates/bevy_retrograde_core && cargo publish --no-verify && cd ../../ && \
        cd crates/bevy_retrograde_physics && cargo publish --no-verify && cd ../../ && \
        cd crates/bevy_retrograde_epaint && cargo publish --no-verify && cd ../../ && \
        cd crates/bevy_retrograde_audio && cargo publish --no-verify && cd ../../ && \
        cd crates/bevy_retrograde_text && cargo publish --no-verify && cd ../../ && \
        cd crates/bevy_retrograde_ui && cargo publish --no-verify && cd ../../ && \
        cd crates/bevy_retrograde_ldtk && cargo publish --no-verify && cd ../../ && \
        cargo publish --no-verify; \
    else \
        echo "You must provide argument 'yes I'm sure'"; \
    fi
