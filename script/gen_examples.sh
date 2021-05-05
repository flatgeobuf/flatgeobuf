set -ex

# Copies the dev examples into the `examples` root and replaces the local 
# fgb libary with the published hosted one

# Change this to point to the latest release
RELEASE_BASE="https://unpkg.com/flatgeobuf@3.7.1"

for file in examples/www_root_dev/examples/leaflet/*.html
do
    base_filename=$(basename $file)
    cat $file | sed "s,/dist/flatgeobuf-geojson.min.js,${RELEASE_BASE}/dist/flatgeobuf-geojson.min.js," > examples/leaflet/$base_filename
done

for file in examples/www_root_dev/examples/openlayers/*.html
do
    base_filename=$(basename $file)
    cat $file | sed "s,/dist/flatgeobuf-ol.min.js,${RELEASE_BASE}/$1," > examples/openlayers/$base_filename
done

