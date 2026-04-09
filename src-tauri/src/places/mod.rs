use std::{collections::HashMap, result, sync};

use anyhow::Result;
use diesel::{
    delete,
    dsl::{insert_into, update},
    query_builder::AsChangeset,
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;
use lazy_static::lazy_static;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use tokio::sync::Mutex;

use crate::{
    app::{ensure_db, DB},
    models::{Layer, Photo, Place, Shape},
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
    ensure_db().await?;
    Ok(layers::table
        .load(DB.lock().await.as_mut().unwrap())
        .await?)
}

pub async fn get_shapes() -> Result<Vec<Shape>> {
    ensure_db().await?;
    Ok(shapes::table
        .load(DB.lock().await.as_mut().unwrap())
        .await?)
}

pub async fn get_places() -> Result<Vec<Place>> {
    ensure_db().await?;
    Ok(places::table
        .load(DB.lock().await.as_mut().unwrap())
        .await?)
}

pub async fn create_layer(id: &String, name: &String, color: &String) -> Result<()> {
    ensure_db().await?;
    insert_into(layers::table)
        .values(Layer {
            id: id.clone(),
            name: name.clone(),
            color: color.clone(),
        })
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

    Ok(())
}

pub async fn delete_layer(
    layer: &String,
    recursive: bool,
    new_layer: &Option<String>,
) -> Result<()> {
    ensure_db().await?;
    if recursive {
        delete(shapes::table.filter(shapes::layer.eq(layer.clone())))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        for place in places::table
            .filter(places::layer.eq(layer.clone()))
            .load::<Place>(DB.lock().await.as_mut().unwrap())
            .await?
        {
            for photo in photos::table
                .filter(photos::location.eq(place.id))
                .load::<Photo>(DB.lock().await.as_mut().unwrap())
                .await?
            {
                update(photos::table.filter(photos::name.eq(photo.name.clone())))
                    .set(photos::location.eq::<Option<String>>(None))
                    .execute(DB.lock().await.as_mut().unwrap())
                    .await?;
            }
        }

        delete(places::table.filter(places::layer.eq(layer.clone())))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    } else if new_layer.is_some() {
        let new_layer = new_layer.as_ref().unwrap();
        update(shapes::table.filter(shapes::layer.eq(layer.clone())))
            .set(shapes::layer.eq(new_layer.clone()))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
        update(places::table.filter(places::layer.eq(layer.clone())))
            .set(places::layer.eq(new_layer))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    }

    delete(layers::table.filter(layers::id.eq(layer)))
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

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
    insert_into(places::table)
        .values(Place {
            id: id.clone(),
            name: name.clone(),
            lat,
            lng,
            layer: layer.clone(),
            category: category.clone(),
            shape: None,
        })
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

    Ok(())
}

pub async fn delete_place(place: &String) -> Result<()> {
    ensure_db().await?;
    for photo in photos::table
        .filter(photos::location.eq(place.clone()))
        .load::<Photo>(DB.lock().await.as_mut().unwrap())
        .await?
    {
        update(photos::table.filter(photos::name.eq(photo.name)))
            .set(photos::location.eq::<Option<String>>(None))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    }

    delete(places::table.filter(places::id.eq(place)))
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

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
    insert_into(shapes::table)
        .values(Shape {
            id: id.clone(),
            shape_type: shape_type.clone(),
            points: points.clone(),
            layer: layer.clone(),
            name: name.clone(),
        })
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

    Ok(())
}

pub async fn delete_shape(shape: &String) -> Result<()> {
    ensure_db().await?;
    for place in places::table
        .filter(places::shape.eq::<Option<String>>(Some(shape.clone())))
        .load::<Place>(DB.lock().await.as_mut().unwrap())
        .await?
    {
        update(places::table.filter(places::id.eq(place.id)))
            .set(places::shape.eq::<Option<String>>(None))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;
    }

    delete(shapes::table.filter(shapes::id.eq(shape)))
        .execute(DB.lock().await.as_mut().unwrap())
        .await?;

    Ok(())
}

impl Layer {
    pub async fn set_layer_color(&self, id: &String, color: &String) -> Result<()> {
        ensure_db().await?;
        update(layers::table.filter(layers::id.eq(id)))
            .set(layers::color.eq(color))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn set_layer_name(&self, id: &String, name: &String) -> Result<()> {
        ensure_db().await?;
        update(layers::table.filter(layers::id.eq(id)))
            .set(layers::name.eq(name))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }
}

impl Serialize for Layer {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let counts_cache = LAYER_COUNTS.lock().unwrap();
        let mut state = serializer.serialize_struct("LayerDto", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("color", &self.color)?;
        state.serialize_field("count", &counts_cache.get(&self.id).unwrap_or(&0))?;
        state.end()
    }
}

impl Place {
    pub async fn set_place_name(&self, id: &String, name: &String) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(places::name.eq(name))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn set_place_category(&self, id: &String, category: &String) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(places::category.eq(category))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn set_place_shape(&self, id: &String, shape: &String) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(places::shape.eq::<Option<String>>(Some(shape.to_string())))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn set_place_layer(&self, place: &String, layer: &String) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(place)))
            .set(places::layer.eq(layer))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn set_place_position(&self, id: &String, lat: f32, lng: f32) -> Result<()> {
        ensure_db().await?;
        update(places::table.filter(places::id.eq(id)))
            .set(PositionUpdate { lat, lng })
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }
}

impl Serialize for Place {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let counts_cache = PLACE_COUNTS.lock().unwrap();
        let mut state = serializer.serialize_struct("PlaceDto", 8)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("lat", &self.lat)?;
        state.serialize_field("lng", &self.lng)?;
        state.serialize_field("layer", &self.layer)?;
        state.serialize_field("category", &self.category)?;
        state.serialize_field("shape", &self.shape)?;
        state.serialize_field("count", &counts_cache.get(&self.id).unwrap_or(&0))?;
        state.end()
    }
}

impl Shape {
    pub async fn set_shape_points(&self, shape: &String, points: &String) -> Result<()> {
        ensure_db().await?;
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::points.eq(points))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn set_shape_layer(&self, shape: &String, layer: &String) -> Result<()> {
        ensure_db().await?;
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::layer.eq(layer))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }

    pub async fn set_shape_name(&self, shape: &String, name: &String) -> Result<()> {
        ensure_db().await?;
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::name.eq(name))
            .execute(DB.lock().await.as_mut().unwrap())
            .await?;

        Ok(())
    }
}
