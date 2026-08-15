#!/usr/bin/env bash
# Mint the REMOTE §9.5 wire's key material (bl-b6fa).
#
# **This is operator tooling, not a protocol.** REMOTE §1.4 rules that
# bootstrapping is explicitly out-of-channel: yog carries no enrollment, pairing
# or account flow, ever, and never mints a certificate of its own. This script
# is the act an operator performs on the boxes, scripted — a private CA and two
# leaves — and it is severable: delete the directory and the wire is off, delete
# this script and the operator runs the same openssl commands by hand.
#
# Usage:
#   make wire-certs                      # loopback, the default port
#   make wire-certs WIRE_HOST=engine.example.com WIRE_PORT=7737
#   make wire-certs WIRE_DIR=/path/to/wire
#
# It refuses to overwrite existing material. A rotation is a deliberate act
# with a blast radius (every seat holding the old CA stops connecting), so it
# is `FORCE=1`, never a silent re-mint.
set -euo pipefail

DIR="${WIRE_DIR:?WIRE_DIR is required (the Makefile passes it)}"
HOST="${WIRE_HOST:-127.0.0.1}"
PORT="${WIRE_PORT:-7737}"
DAYS="${WIRE_DAYS:-825}"
FORCE="${FORCE:-}"

command -v openssl >/dev/null || { echo "wire-certs: openssl is not installed" >&2; exit 1; }

if [ -e "$DIR/ca.pem" ] && [ -z "$FORCE" ]; then
  echo "wire-certs: $DIR already holds material; rotating distrusts every issued" >&2
  echo "            certificate. Re-run with FORCE=1 if that is what you mean." >&2
  exit 1
fi

# A server certificate is verified against the NAME a seat dialled, so the SAN
# has to say which kind of name that is. The address file and the SAN are
# derived from the same HOST for exactly that reason — two spellings of one
# host is the drift this whole design removes from the boundary.
case "$HOST" in
  *[!0-9.]*) SAN="DNS:$HOST" ;;
  *)         SAN="IP:$HOST" ;;
esac

mkdir -p "$DIR"
chmod 700 "$DIR"
umask 077

leaf() { # leaf() <name> <san> <eku>
  openssl req -new -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -sha256 \
    -subj "/CN=yog-$1" -addext "subjectAltName=$2" -addext "extendedKeyUsage=$3" \
    -keyout "$DIR/$1.key" -out "$DIR/$1.csr" 2>/dev/null
  openssl x509 -req -sha256 -days "$DAYS" -copy_extensions copy \
    -in "$DIR/$1.csr" -CA "$DIR/ca.pem" -CAkey "$DIR/ca.key" \
    -out "$DIR/$1.pem" 2>/dev/null
  rm -f "$DIR/$1.csr"
  chmod 600 "$DIR/$1.key"
}

openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -sha256 \
  -days "$DAYS" -subj "/CN=yog-ca" \
  -keyout "$DIR/ca.key" -out "$DIR/ca.pem" 2>/dev/null
chmod 600 "$DIR/ca.key"

leaf server "$SAN" serverAuth
leaf client "DNS:yog-client" clientAuth

printf '%s:%s\n' "$HOST" "$PORT" >"$DIR/address"

# The CA key is the whole trust root: it is what issues the NEXT client, and
# nothing but issuance needs it. Say where it is; never print any of it.
echo "wire-certs: minted into $DIR (ca.pem, server.pem/key, client.pem/key, address)"
echo "            the engine binds and a local seat dials $HOST:$PORT"
echo "            issue another client with: openssl req … -CA $DIR/ca.pem -CAkey $DIR/ca.key"
