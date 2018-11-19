FROM elixir:1.7.2

LABEL MAINTAINER="Casey Primozic <me@ameo.link>"

# Install NodeJS, Yarn, and build deps
RUN apt-get update
RUN apt-get install -y apt-transport-https build-essential cmake git
RUN curl -sL https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && \
  echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list && \
  apt-get update && apt-get install -y yarn
RUN curl -sL https://deb.nodesource.com/setup_10.x | bash
RUN apt-get install -y nodejs

# Install `wasm-opt`
RUN git clone https://github.com/WebAssembly/binaryen.git /tmp/binaryen
WORKDIR /tmp/binaryen
RUN cmake . && make && make install

# Install rust
RUN curl https://sh.rustup.rs/ -sSf | \
  sh -s -- -y --default-toolchain nightly-2018-10-15 && \
  PATH=$HOME/.cargo/bin:$PATH rustup target add wasm32-unknown-unknown --toolchain nightly-2018-10-15

# Install wasm-bindgen
RUN PATH=$HOME/.cargo/bin:$PATH cargo install wasm-bindgen-cli

ADD . /app

WORKDIR /app/frontend

# Build frontend and optimize emitted WebAssembly blob
RUN yarn
RUN PATH=$HOME/.cargo/bin:$PATH bash build_all.sh

RUN cp /app/frontend/dist/* /app/backend/priv/static
RUN cp /app/frontend/dist/index.html /app/backend/lib/backend_web/templates/page/index.html.eex

WORKDIR /app/backend

RUN mix local.hex --force
RUN mix local.rebar --force
RUN mix deps.get
RUN mix phx.digest

RUN ln -s $HOME/.cargo/bin/cargo /usr/local/bin/cargo
RUN ln -s $HOME/.cargo/bin/rustc /usr/local/bin/rustc

RUN mix compile

# Start Elixir Phoenix server which serves backend + frontend
ENV PORT=3699
ENV MIX_ENV=prod
ENV PATH=$HOME/.cargo/bin:${PATH}
CMD ["iex", "-S", "mix", "phx.server"]
