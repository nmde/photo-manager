# photo-manager

Organize photos and videos with an advanced tagging system and geolocation.

## Features
- Tags:
  - Tag photos for easy categorization
  - Color-code tags
  - View which tags are used the most
  - Add prerequisite, corequisite, and incompatible tags for advanced organization
  - Advanced search by tags allows you to include and exclude tags from view, and search using multiple tags
- Geolocation:
  - Place photos on a map to mark where they're taken
  - Geolocation is not written to the file, to avoid accidental privay violations
  - View a heatmap of where your photos are most commonly taken
  - Click on the map to filter your collection by location
- Supports RAW photo formats
- Supports videos, and has an embedded video player

# Installation
- Install [Rust](https://www.rust-lang.org/learn/get-started)
- Install [ImageMagick](https://imagemagick.org/script/download.php)
- Install [ffmpeg](https://ffmpeg.org/download.html)
- Initialize Nuxt by running `yarn nuxt dev`
- Launch the program by running `yarn tauri:dev`
