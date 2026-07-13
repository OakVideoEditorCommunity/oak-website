FROM python:3.12-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY build-docs.py /usr/local/bin/build-docs
RUN chmod +x /usr/local/bin/build-docs

ENV DOCS_SRC_DIR=/data/oak-docs
ENV DOCS_OUT_DIR=/data/docs-html

CMD ["build-docs"]
