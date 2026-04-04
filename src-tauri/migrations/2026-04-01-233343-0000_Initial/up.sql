CREATE TABLE
    "layers" (
        "id" TEXT NOT NULL UNIQUE,
        "name" TEXT NOT NULL,
        "color" TEXT NOT NULL,
        PRIMARY KEY ("id")
    );

CREATE TABLE
    "people" (
        "id" TEXT NOT NULL UNIQUE,
        "name" TEXT NOT NULL,
        "photo" TEXT,
        "category" TEXT NOT NULL,
        PRIMARY KEY ("id")
    );

CREATE TABLE
    "people_categories" (
        "id" TEXT NOT NULL UNIQUE,
        "name" TEXT NOT NULL,
        "color" TEXT NOT NULL,
        PRIMARY KEY ("id")
    );

CREATE TABLE
    "photos" (
        "path" TEXT NOT NULL UNIQUE,
        "title" TEXT,
        "description" TEXT,
        "tags" TEXT,
        "is_duplicate" INTEGER,
        "rating" INTEGER,
        "location" TEXT,
        "thumbnail" TEXT,
        "photo_group" TEXT,
        "date" TEXT,
        "people" TEXT,
        "hide_thumbnail" INTEGER,
        "photographer" TEXT,
        PRIMARY KEY ("path")
    );

CREATE TABLE
    "photo_groups" (
        "id" TEXT NOT NULL UNIQUE,
        "name" TEXT NOT NULL,
        PRIMARY KEY ("id")
    );

CREATE TABLE
    "places" (
        "id" TEXT NOT NULL UNIQUE,
        "name" TEXT NOT NULL,
        "lat" REAL NOT NULL,
        "lng" REAL NOT NULL,
        "layer" TEXT NOT NULL,
        "category" TEXT NOT NULL,
        "shape" TEXT,
        PRIMARY KEY ("id")
    );

CREATE TABLE
    "settings" (
        "setting" TEXT NOT NULL UNIQUE,
        "value" INTEGER NOT NULL,
        PRIMARY KEY ("setting")
    );

CREATE TABLE
    "shapes" (
        "id" TEXT NOT NULL UNIQUE,
        "shape_type" TEXT NOT NULL,
        "points" TEXT NOT NULL,
        "layer" TEXT NOT NULL,
        "name" TEXT NOT NULL,
        PRIMARY KEY ("id")
    );

CREATE TABLE
    "tags" (
        "name" TEXT NOT NULL UNIQUE,
        "color" TEXT NOT NULL,
        "prereqs" TEXT,
        "coreqs" TEXT,
        "incompatible" TEXT,
        PRIMARY KEY ("name")
    );