FROM python:3.12-slim

# Install git so the builder can optionally clone documentation from a remote repository.
RUN apt-get update && apt-get install -y --no-install-recommends git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY build-docs.py /usr/local/bin/build-docs
RUN chmod +x /usr/local/bin/build-docs

ENV DOCS_SRC_DIR=/data/oak-docs
ENV DOCS_OUT_DIR=/data/docs-html
# Optional comma-separated git refs to build as versions (first = latest).
# Default empty: build all semver tags, or a single "latest" version if none.
ENV DOCS_VERSIONS=""

CMD ["build-docs"]
