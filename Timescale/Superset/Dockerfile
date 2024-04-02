FROM apache/superset:4.0.0rc2
USER root

ARG ADMIN_NAME=admin
ARG ADMIN_PASS=admin
ARG DATABASE_DISPLAY_NAME
ARG DATABASE_DB
ARG DATABASE_HOST
ARG DATABASE_USER
ARG DATABASE_PASSWORD
ARG DATABASE_PORT
ARG DATABASE_DIALECT

ENV PYTHONPATH=/app/pythonpath
ENV SUPERSET_LOAD_EXAMPLES=no
# ENV FLASK_ENV=production
# ENV SUPERSET_ENV=production
# ENV SECRET_KEY=qV4ZCA3qMX9f9xbJ1ZR92eY4WfJkvegpsqEfBL5uNIJTViK2Q2Snmj0s

COPY ./superset_config.py /app/pythonpath/superset_config.py

# postgres driver
RUN pip install psycopg2
RUN superset fab create-admin --username ${ADMIN_NAME} --password ${ADMIN_PASS} --firstname admin --lastname admin --email admin@superset.com
RUN superset db upgrade
RUN superset init
RUN superset set-database-uri -u "${DATABASE_DIALECT}://${DATABASE_USER}:${DATABASE_PASSWORD}@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_DB}" -d "${DATABASE_DISPLAY_NAME}"
