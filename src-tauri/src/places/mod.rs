use anyhow::Result;
use diesel::{
    delete,
    dsl::{insert_into, update},
    query_builder::AsChangeset,
    ExpressionMethods, QueryDsl,
};
use diesel_async::RunQueryDsl;

use crate::{
    models::{Layer, Photo, Place, Shape},
    schema::{layers, photos, places, shapes},
    PhotoManager,
};

pub mod api;

#[derive(AsChangeset)]
#[diesel(table_name = places)]
struct PositionUpdate {
    lat: f32,
    lng: f32,
}

impl PhotoManager {
    pub async fn get_layers(&mut self) -> Result<Vec<Layer>> {
        Ok(layers::table.load(&mut self.db).await?)
    }

    pub async fn get_shapes(&mut self) -> Result<Vec<Shape>> {
        Ok(shapes::table.load(&mut self.db).await?)
    }

    pub async fn get_places(&mut self) -> Result<Vec<Place>> {
        Ok(places::table.load(&mut self.db).await?)
    }

    pub async fn create_layer(&mut self, id: &String, name: &String, color: &String) -> Result<()> {
        insert_into(layers::table)
            .values(Layer {
                id: id.clone(),
                name: name.clone(),
                color: color.clone(),
            })
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_layer_color(&mut self, id: &String, color: &String) -> Result<()> {
        update(layers::table.filter(layers::id.eq(id)))
            .set(layers::color.eq(color))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn delete_layer(
        &mut self,
        layer: &String,
        recursive: bool,
        new_layer: &Option<String>,
    ) -> Result<()> {
        if recursive {
            delete(shapes::table.filter(shapes::layer.eq(layer.clone())))
                .execute(&mut self.db)
                .await?;

            for place in places::table
                .filter(places::layer.eq(layer.clone()))
                .load::<Place>(&mut self.db)
                .await?
            {
                for photo in photos::table
                    .filter(photos::location.eq(place.id))
                    .load::<Photo>(&mut self.db)
                    .await?
                {
                    update(photos::table.filter(photos::name.eq(photo.name.clone())))
                        .set(photos::location.eq::<Option<String>>(None))
                        .execute(&mut self.db)
                        .await?;
                }
            }

            delete(places::table.filter(places::layer.eq(layer.clone())))
                .execute(&mut self.db)
                .await?;
        } else if new_layer.is_some() {
            let new_layer = new_layer.as_ref().unwrap();
            update(shapes::table.filter(shapes::layer.eq(layer.clone())))
                .set(shapes::layer.eq(new_layer.clone()))
                .execute(&mut self.db)
                .await?;
            update(places::table.filter(places::layer.eq(layer.clone())))
                .set(places::layer.eq(new_layer))
                .execute(&mut self.db)
                .await?;
        }

        delete(layers::table.filter(layers::id.eq(layer)))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn create_place(
        &mut self,
        id: String,
        name: &String,
        lat: f32,
        lng: f32,
        layer: &String,
        category: &String,
    ) -> Result<()> {
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
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_place_name(&mut self, id: &String, name: &String) -> Result<()> {
        update(places::table.filter(places::id.eq(id)))
            .set(places::name.eq(name))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_place_category(&mut self, id: &String, category: &String) -> Result<()> {
        update(places::table.filter(places::id.eq(id)))
            .set(places::category.eq(category))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_place_shape(&mut self, id: &String, shape: &String) -> Result<()> {
        update(places::table.filter(places::id.eq(id)))
            .set(places::shape.eq::<Option<String>>(Some(shape.to_string())))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_place_layer(&mut self, place: &String, layer: &String) -> Result<()> {
        update(places::table.filter(places::id.eq(place)))
            .set(places::layer.eq(layer))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_place_position(&mut self, id: &String, lat: f32, lng: f32) -> Result<()> {
        update(places::table.filter(places::id.eq(id)))
            .set(PositionUpdate { lat, lng })
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn delete_place(&mut self, place: &String) -> Result<()> {
        for photo in photos::table
            .filter(photos::location.eq(place.clone()))
            .load::<Photo>(&mut self.db)
            .await?
        {
            update(photos::table.filter(photos::name.eq(photo.name)))
                .set(photos::location.eq::<Option<String>>(None))
                .execute(&mut self.db)
                .await?;
        }

        delete(places::table.filter(places::id.eq(place)))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn create_shape(
        &mut self,
        id: &String,
        shape_type: &String,
        points: &String,
        layer: &String,
        name: &String,
    ) -> Result<()> {
        insert_into(shapes::table)
            .values(Shape {
                id: id.clone(),
                shape_type: shape_type.clone(),
                points: points.clone(),
                layer: layer.clone(),
                name: name.clone(),
            })
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_shape_points(&mut self, shape: &String, points: &String) -> Result<()> {
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::points.eq(points))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_shape_layer(&mut self, shape: &String, layer: &String) -> Result<()> {
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::layer.eq(layer))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn set_shape_name(&mut self, shape: &String, name: &String) -> Result<()> {
        update(shapes::table.filter(shapes::id.eq(shape)))
            .set(shapes::name.eq(name))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn delete_shape(&mut self, shape: &String) -> Result<()> {
        for place in places::table
            .filter(places::shape.eq::<Option<String>>(Some(shape.clone())))
            .load::<Place>(&mut self.db)
            .await?
        {
            update(places::table.filter(places::id.eq(place.id)))
                .set(places::shape.eq::<Option<String>>(None))
                .execute(&mut self.db)
                .await?;
        }

        delete(shapes::table.filter(shapes::id.eq(shape)))
            .execute(&mut self.db)
            .await?;

        Ok(())
    }

    pub async fn count_places_in_layer(&mut self, layer: &String) -> Result<usize> {
        Ok(places::table
            .filter(places::layer.eq(layer))
            .load::<Place>(&mut self.db)
            .await?
            .len())
    }

    pub async fn count_photos_in_place(&mut self, place: &String) -> Result<usize> {
        Ok(photos::table
            .filter(photos::location.eq(place))
            .load::<Photo>(&mut self.db)
            .await?
            .len())
    }
}
