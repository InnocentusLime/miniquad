FROM rust:1.93.1-bookworm AS build

RUN <<EOF
    cargo install -f wasm-bindgen-cli --version 0.2.113 &&\
    rustup target add wasm32-unknown-unknown
EOF
RUN mkdir /dist
COPY /examples/pages /dist
COPY /examples/assets/ /dist/assets/
ADD . /project/
RUN <<EOF
    cd /project
    cargo build --example basic_img --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example batcher --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example blobs --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example egui --features egui --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example input --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example offscreen --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example post_processing --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example quad --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example raw_input --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example shape_batcher --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example sprite_batch --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example triangle_color4b --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example triangle_verbose --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example triangle --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example window_conf --target wasm32-unknown-unknown --profile wasm-release
    cargo build --example timing --target wasm32-unknown-unknown --profile wasm-release
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/basic_img.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/batcher.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/blobs.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/egui.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/input.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/offscreen.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/post_processing.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/quad.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/raw_input.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/shape_batcher.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/sprite_batch.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/triangle_color4b.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/triangle_verbose.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/triangle.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/window_conf.wasm
    wasm-bindgen --target web --out-dir /dist ./target/wasm32-unknown-unknown/wasm-release/examples/timing.wasm
EOF

FROM httpd:trixie 
COPY --from=build /dist /usr/local/apache2/htdocs/ 