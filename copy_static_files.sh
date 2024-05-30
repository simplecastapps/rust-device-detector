# Description: Copy the static files from the original project to this project
# Usage: ./copy_static_files.sh /path/to/original/device/detector
#
# Note, there will be many source files that have to be modified including
# lists of known browsers and operating systems to keep this library up to date.

ORIGIN_DIR="$1"

cp $ORIGIN_DIR/regexes/*.yml regexes/
cp $ORIGIN_DIR/regexes/device/*.yml regexes/device/
cp $ORIGIN_DIR/regexes/client/*.yml regexes/client/
cp $ORIGIN_DIR/regexes/client/hints/*.yml regexes/client/hints/

cp $ORIGIN_DIR/Tests/fixtures/*.yml tests/data/fixtures/
cp $ORIGIN_DIR/Tests/Parser/fixtures/*.yml tests/data/fixtures/parser/
cp $ORIGIN_DIR/Tests/Parser/Client/fixtures/*.yml tests/data/fixtures/parser/client/
cp $ORIGIN_DIR/Tests/Parser/Device/fixtures/*.yml tests/data/fixtures/parser/device/

