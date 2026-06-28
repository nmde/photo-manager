-- This file is just for converting my personal projects to the v1 database schema, should be removed after the release
UPDATE photos
SET
    name = replace (
        replace (
            replace (
                replace (
                    replace (
                        replace (substr (asset_path, 25), "%3A", ":"),
                        "%5C",
                        "\"
                    ),
                    "%5B",
                    "["
                ),
                "%5D",
                "]"
            ),
            "%20",
            " "
        ),
        "%2C",
        ","
    )