use std::{collections::HashMap, sync};

use anyhow::Result;
use diesel::{
    delete,
    dsl::{insert_into, update},
    query_builder::AsChangeset,
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    app::{ensure_db, DB},
    models::{Layer, Photo, Place, Shape},
    photos::PHOTOS,
    schema::{layers, photos, places, shapes},
};

pub mod api;

lazy_static! {
    pub static ref LAYERS: Mutex<HashMap<String, Layer>> = Mutex::new(HashMap::new());
    pub static ref SHAPES: Mutex<HashMap<String, Shape>> = Mutex::new(HashMap::new());
    pub static ref PLACES: Mutex<HashMap<String, Place>> = Mutex::new(HashMap::new());
    pub static ref LAYER_COUNTS: sync::Mutex<HashMap<String, usize>> =
        sync::Mutex::new(HashMap::new());
    pub static ref PLACE_COUNTS: sync::Mutex<HashMap<String, usize>> =
        sync::Mutex::new(HashMap::new());
}

#[derive(AsChangeset)]
#[diesel(table_name = places)]
struct PositionUpdate {
    lat: f32,
    lng: f32,
}

pub async fn get_layers() -> Result<Vec<Layer>> {
    Ok(LAYERS
        .lock()
        .await
        .to_owned()
        .values()
        .map(|x| x.clone())
        .collect::<Vec<Layer>>())
}

pub async fn get_shapes() -> Result<Vec<Shape>> {
    Ok(SHAPES
        .lock()
        .await
        .to_owned()
        .values()
        .map(|x| x.clone())
        .collect::<Vec<Shape>>())
}

pub async fn get_places() -> Result<Vec<Place>> {
    Ok(PLACES
        .lock()
        .await
        .to_owned()
        .values()
        .map(|x| x.clone())
        .collect::<Vec<Place>>())
}

pub async fn create_layer(id: &String, name: &String, color: &String) -> Result<()> {
    ensure_db().await?;
    let new_layer = Layer {
        id: id.clone(),
        name: name.clone(),
        color: color.clone(),
    };
    insert_into(layers::table)
        .values(new_layer.clone())
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;
    LAYERS.lock().await.insert(id.clone(), new_layer);
    LAYER_COUNTS.lock().unwrap().insert(id.clone(), 0);

    Ok(())
}

pub async fn delete_layer(
    layer: &String,
    recursive: bool,
    new_layer: &Option<String>,
) -> Result<()> {
    ensure_db().await?;
    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();

    if recursive {
        delete(shapes::table.filter(shapes::layer.eq(layer.clone())))
            .execute(conn)
            .await?;
        SHAPES.lock().await.retain(|_, shape| shape.layer != *layer);

        let mut photos = PHOTOS.lock().await;
        for place in places::table
            .filter(places::layer.eq(layer.clone()))
            .load::<Place>(conn)
            .await?
        {
            for photo in photos::table
                .filter(photos::location.eq(place.id))
                .load::<Photo>(conn)
                .await?
            {
                update(photos::table.filter(photos::name.eq(photo.name.clone())))
                    .set(photos::location.eq::<Option<String>>(None))
                    .execute(conn)
                    .await?;
                photos.get_mut(&photo.name).unwrap().location = None;
            }
        }

        delete(places::table.filter(places::layer.eq(layer.clone())))
            .execute(conn)
            .await?;
        PLACES.lock().await.retain(|_, place| place.layer != *layer);
    } else if new_layer.is_some() {
        let new_layer = new_layer.as_ref().unwrap();
        update(shapes::table.filter(shapes::layer.eq(layer.clone())))
            .set(shapes::layer.eq(new_layer.clone()))
            .execute(conn)
            .await?;
        update(places::table.filter(places::layer.eq(layer.clone())))
            .set(places::layer.eq(new_layer))
            .execute(conn)
            .await?;
        let mut shapes = SHAPES.lock().await;
        for (_, shape) in shapes.iter_mut() {
            if shape.layer == *layer {
                shape.layer = new_layer.clone();
            }
        }
        let mut places = PLACES.lock().await;
        for (_, place) in places.iter_mut() {
            if place.layer == *layer {
                place.layer = new_layer.clone();
            }
        }
    }

    delete(layers::table.filter(layers::id.eq(layer)))
        .execute(conn)
        .await?;
    LAYERS.lock().await.remove(layer);
    LAYER_COUNTS.lock().unwrap().remove(layer);

    Ok(())
}

pub async fn create_place(
    id: String,
    name: &String,
    lat: f32,
    lng: f32,
    layer: &String,
    category: &String,
) -> Result<()> {
    ensure_db().await?;

    let new_place = Place {
        id: id.clone(),
        name: name.clone(),
        lat,
        lng,
        layer: layer.clone(),
        category: category.clone(),
        shape: None,
    };
    insert_into(places::table)
        .values(new_place.clone())
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;
    PLACES.lock().await.insert(id.clone(), new_place);

    Ok(())
}

pub async fn delete_place(place: &String) -> Result<()> {
    ensure_db().await?;
    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();
    for photo in photos::table
        .filter(photos::location.eq(place.clone()))
        .load::<Photo>(conn)
        .await?
    {
        update(photos::table.filter(photos::name.eq(photo.name)))
            .set(photos::location.eq::<Option<String>>(None))
            .execute(conn)
            .await?;
    }
    let mut photos = PHOTOS.lock().await;
    for (_, photo) in photos.iter_mut() {
        if photo.location.as_ref().unwrap_or(&String::new()) == place {
            photo.location = None;
        }
    }

    delete(places::table.filter(places::id.eq(place)))
        .execute(conn)
        .await?;
    PLACES.lock().await.remove(place);

    Ok(())
}

pub async fn create_shape(
    id: &String,
    shape_type: &String,
    points: &String,
    layer: &String,
    name: &String,
) -> Result<()> {
    ensure_db().await?;

    let new_shape = Shape {
        id: id.clone(),
        shape_type: shape_type.clone(),
        points: points.clone(),
        layer: layer.clone(),
        name: name.clone(),
    };
    insert_into(shapes::table)
        .values(new_shape.clone())
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;
    SHAPES.lock().await.insert(id.clone(), new_shape);

    Ok(())
}

pub async fn delete_shape(shape: &String) -> Result<()> {
    ensure_db().await?;
    let mut conn = DB.lock().await;
    let conn = conn.as_mut().unwrap();
    for place in places::table
        .filter(places::shape.eq::<Option<String>>(Some(shape.clone())))
        .load::<Place>(conn)
        .await?
    {
        update(places::table.filter(places::id.eq(place.id)))
            .set(places::shape.eq::<Option<String>>(None))
            .execute(conn)
            .await?;
    }
    let mut places = PLACES.lock().await;
    for (_, place) in places.iter_mut() {
        if place.shape.as_ref().unwrap_or(&String::new()) == shape {
            place.shape = None;
        }
    }

    delete(shapes::table.filter(shapes::id.eq(shape)))
        .execute(conn)
        .await?;
    SHAPES.lock().await.remove(shape);

    Ok(())
}

impl Layer {
    pub async fn set_layer_color(&mut self, id: &String, color: &String) -> Result<()> {
        ensure_db().await?;
        update(layers::table.filter(layers::id.eq(id)))
            .set(layers::color.eq(color))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.color = color.clone();

        Ok(())
    }

    pub async fn set_layer_name(&mut self, id: &String, name: &String) -> Result<()> {
        ensure_db().await?;
        update(layers::table.filter(layers::id.eq(id)))
            .set(layers::name.eq(name))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.name = name.clone();

        Ok(())
    }
}

#[derive(Serialize)]
pub struct LayerDto {
    pub id: String,
    pub name: String,
    pub color: String,
    pub count: usize,
}

impl From<&Layer> for LayerDto {
    fn from(value: &Layer) -> Self {
        let counts_cache = LAYER_COUNTS.lock().unwrap();
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            color: value.color.clone(),
            count: counts_cache.get(&value.id).unwrap_or(&0).clone(),
        }
    }
}

impl Place {
    pub async fn set_place_name(&mut self, id: &String, name: &String) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(places::name.eq(name))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.name = name.clone();

        Ok(())
    }

    pub async fn set_place_category(&mut self, id: &String, category: &String) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(places::category.eq(category))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.category = category.clone();

        Ok(())
    }

    pub async fn set_place_shape(&mut self, id: &String, shape: &Option<String>) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(places::shape.eq(shape))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.shape = shape.clone();

        Ok(())
    }

    pub async fn set_place_layer(&mut self, place: &String, layer: &String) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(place)))
            .set(places::layer.eq(layer))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.layer = layer.clone();

        Ok(())
    }

    pub async fn set_place_position(&mut self, id: &String, lat: f32, lng: f32) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(PositionUpdate { lat, lng })
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.lat = lat;
        self.lng = lng;

        Ok(())
    }
}

#[derive(Serialize)]
pub struct PlaceDto {
    pub id: String,
    pub name: String,
    pub lat: f32,
    pub lng: f32,
    pub layer: String,
    pub category: String,
    pub shape: Option<String>,
    pub count: usize,
}

impl From<&Place> for PlaceDto {
    fn from(value: &Place) -> Self {
        let counts_cache = PLACE_COUNTS.lock().unwrap();
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            lat: value.lat,
            lng: value.lng,
            layer: value.layer.clone(),
            category: value.category.clone(),
            shape: value.shape.clone(),
            count: counts_cache.get(&value.id).unwrap_or(&0).clone(),
        }
    }
}

impl Shape {
    pub async fn set_shape_points(&mut self, shape: &String, points: &String) -> Result<()> {
        ensure_db().await?;
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::points.eq(points))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.points = points.clone();

        Ok(())
    }

    pub async fn set_shape_layer(&mut self, shape: &String, layer: &String) -> Result<()> {
        ensure_db().await?;
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::layer.eq(layer))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.layer = layer.clone();

        Ok(())
    }

    pub async fn set_shape_name(&mut self, shape: &String, name: &String) -> Result<()> {
        ensure_db().await?;
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::name.eq(name))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        self.name = name.clone();

        Ok(())
    }
}
