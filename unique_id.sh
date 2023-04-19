ENV_VARS="$(cat .env | awk '!/^\s*#/' | awk '!/^\s*$/')"

eval "$(
  printf '%s\n' "$ENV_VARS" | while IFS='' read -r line; do
    key=$(printf '%s\n' "$line"| sed 's/"/\\"/g' | cut -d '=' -f 1)
    value=$(printf '%s\n' "$line" | cut -d '=' -f 2- | sed 's/"/\\\"/g')
    printf '%s\n' "export $key=\"$value\""
  done
)"

flag=""
dir_name="debug"
if [ "$RELEASE" = true ]
then
  flag="--release"
  dir_name="release"
fi

cargo build $(flag) --bin unique_ids
$MAELSTORM_DIR/maelstrom test -w unique-ids --bin target/$dir_name/unique_ids --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition