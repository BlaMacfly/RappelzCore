#!/usr/bin/env bash
# Importe le schéma AzerothCore (sql/base) puis les ajouts RappelzCore (sql/custom) dans les
# trois bases. Usage : tools/sql/import_all.sh [hôte] [port] [utilisateur] [mot_de_passe]
# Le client `mysql`/`mariadb` doit être dans le PATH (ou MYSQL=chemin/vers/mariadb.exe).
set -euo pipefail
cd "$(dirname "$0")/../.."
HOST=${1:-127.0.0.1}
PORT=${2:-3306}
USER=${3:-acore}
PASS=${4:-acore}
MYSQL=${MYSQL:-$(command -v mariadb || command -v mysql)}

for pair in db_auth:acore_auth db_characters:acore_characters db_world:acore_world; do
    dir=${pair%%:*}
    db=${pair##*:}
    echo ">> $db"
    cat sql/base/$dir/*.sql $(ls sql/custom/$dir/*.sql 2>/dev/null) | "$MYSQL" -h"$HOST" -P"$PORT" -u"$USER" -p"$PASS" "$db"
done
echo "Schéma importé. Données du monde : rz-dbimport --data-dir <DataDir> --apply \"$HOST;$PORT;$USER;$PASS;acore_world\""
