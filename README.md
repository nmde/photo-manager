# photo-manager

Organize photos and videos with an advanced tagging system and geolocation.

## Photo Tagging
- Tag photos for easy categorization
- Color-code tags
- View which tags are used the most
- Rate photos and see which tags have the greatest influence on rating
- Add prerequisite, corequisite, and incompatible tags for advanced organization
- Advanced search by tags allows you to include and exclude tags from view, and search using multiple tags with AND or OR logic
- Group similar photos
- Add titles, descriptions, and dates to phoos
- Supports RAW photo formats
- Supports videos, and has an embedded video player

## Custom Mapping
- Define places with custom titles, notes, and icons
- Organize places with layers and set custom colors for the places in each layer
- Assign photos to places to search by location
- Geolocation is not written to the file, to avoid accidental privacy violations
- View a heatmap of where your photos are most commonly taken
- Draw on the map with lines and polygons

## Calendar / Journal View
- View photos taken each day on the calendar
- Keep journal entries including mood, activities, and steps taken
- Encrypt journal entries in the database and application memory

## People Tagging
- Tag people in photos
- Tag who took a photo
- View photos of or taken by a particular person

# Search Syntax
The following search terms are implemented. Any search term can be negated by prefixing it with "-".
- at:[location] - At the specified location
- only:[person] - Only the specified person (and no one else)
- by:[person] - Taken by the specified person
- has:(rating|photographer|location|people|tags) - Photos that have a value for the specified field
- name:[name] - The photo's name (file path) contains the specified text
- rating(<=|>=|<|>|=)[rating] - Photos with a rating compared to the given value
- of:[person] - Photos that include the specified person
- date(<=|>=|<|>|=)[date] - Photos with a date compared to the given value
- is:(raw|video) - Only RAW or video files

# Installation
- Install [Rust](https://www.rust-lang.org/learn/get-started)
- Install [ImageMagick](https://imagemagick.org/script/download.php)
- Install [ffmpeg](https://ffmpeg.org/download.html)
- Initialize Nuxt by running `yarn add nuxt` then `yarn nuxt dev`
- Launch the program by running `yarn tauri dev`
