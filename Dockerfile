FROM ubuntu:22.04

# install deps
RUN apt-get update && apt-get install -y --no-install-recommends file ca-certificates wget git pkg-config libasound2 libjack-jackd2-dev libasound2-dev libssl-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-0 libgtk-3-dev curl build-essential

# fix cert
RUN update-ca-certificates

# install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# configure shell
ENV PATH="/root/.cargo/bin:${PATH}"

# install cargo-appimage
RUN cargo install cargo-appimage # file package is required by appimagetool

# switch to nightly
RUN rustup default nightly

# latest version
RUN rustup update

# download appimagetool
RUN wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-$(uname -m).AppImage -O /usr/local/bin/appimagetool

# apply hacks
RUN chmod +x /usr/local/bin/appimagetool # Path appimagetool magic byte: https://github.com/AppImage/pkg2appimage/issues/373#issuecomment-495754112

# apply hacks
RUN sed -i 's|AI\x02|\x00\x00\x00|' /usr/local/bin/appimagetool # Use appimagetool without fuse: https://github.com/AppImage/AppImageKit/wiki/FUSE#docker

# get megra
RUN git clone https://github.com/the-drunk-coder/megra.rs

# latest stuff
RUN cd megra.rs && cargo update

# build appimage
RUN cd megra.rs && APPIMAGE_EXTRACT_AND_RUN=1 cargo appimage
