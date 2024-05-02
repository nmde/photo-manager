import { Loader } from '@googlemaps/js-api-loader';
import { color as d3color } from 'd3-color';
import { EventEmitter } from 'ee-ts';
import { Shape, type ShapeType } from './Shape';

export type Position = {
  lat: number;
  lng: number;
};

export type Marker = {
  el: google.maps.marker.AdvancedMarkerElement;
  position: Position;
  count: number;
};

export const icons = {
  hospital: 'mdi-hospital',
};

export type PlaceType = keyof typeof icons;

/**
 * Helper method to get lat,lng as a string.
 * @param param0 - The location.
 * @returns The location string.
 */
export function locToString(location?: { lat: number; lng: number }) {
  if (location) {
    return `${location.lat},${location.lng}`;
  }
  return '';
}

/**
 * Helper method to get lat,lng from a string.
 * @param str - The string.
 * @returns The location object.
 */
export function stringToLoc(str: string) {
  const split = str.split(',').map((x) => Number(x));
  return {
    lat: split[0],
    lng: split[1],
  };
}

/**
 * Provides functions for working with google maps.
 */
export class Map extends EventEmitter<{
  markerClicked: (pos: Position) => void;
  markerCreated: (pos: Position) => void;
  click: (pos: Position) => void;
  dblclick: (pos: Position) => void;
  shapeUpdate: (newPath: google.maps.MVCArray<google.maps.LatLng>) => void;
}> {
  private heatmap!: google.maps.visualization.HeatmapLayer;

  private map!: google.maps.Map;

  private mapsLibrary!: google.maps.MapsLibrary;

  private markers: Record<string, Marker> = {};

  private markerLibrary!: google.maps.MarkerLibrary;

  private maxCount = 0;

  private shapes: Record<string, google.maps.Polyline | google.maps.Polygon> = {};

  private visualizationLibrary!: google.maps.VisualizationLibrary;

  /**
   * Creates the heatmap visualization.
   */
  public createHeatmap() {
    const bins: number[] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1];
    const binSize = this.maxCount / 10;
    this.heatmap = new this.visualizationLibrary.HeatmapLayer({
      data: Object.values(this.markers).map((v) => {
        let bin = 0;
        while (v.count < binSize * bin && bin < 11) {
          bin += 1;
        }
        return {
          location: new google.maps.LatLng(v.position.lat, v.position.lng),
          weight: bins[bin],
        };
      }),
      dissipating: true,
      radius: 20,
      maxIntensity: 1,
    });
  }

  /**
   * Places a marker on the map.
   * @param pos - The position to place the marker at.
   * @param icon - An icon to use for the marker.
   * @param color - The color of the marker.
   * @param title - The title of the marker.
   */
  public createMarker(pos: string, icon?: keyof typeof icons, color?: string, title?: string) {
    if (!this.markers[pos]) {
      const position = stringToLoc(pos);
      const marker: google.maps.marker.AdvancedMarkerElementOptions = {
        map: this.map,
        position,
        title,
      };
      if (typeof icon === 'string' && typeof color === 'string') {
        const i = document.createElement('div');
        i.innerHTML = `<i class="mdi ${icons[icon]}"></i>`;
        marker.content = new this.markerLibrary.PinElement({
          glyph: i,
          background: color,
          borderColor: d3color(color)?.darker(0.15).toString(),
        }).element;
      }
      this.markers[pos] = {
        el: new this.markerLibrary.AdvancedMarkerElement(marker),
        position,
        count: 1, // TODO
      };
      google.maps.event.addListener(this.markers[pos].el, 'click', () => {
        this.emit('markerClicked', position);
      });
      this.map.setCenter(position);
      /**
      if (count > this.maxCount) {
        this.maxCount = count;
      }
       */
    }
  }

  /**
   * Creates a shape on the map.
   * @param type - The type of shape.
   * @param points - The points of the shape.
   * @param color
   * @param title
   */
  public createShape(type: ShapeType, points: Position[], color: string) {
    let shape;
    if (type === 'line') {
      shape = new this.mapsLibrary.Polyline({
        path: points,
        geodesic: true,
        strokeColor: color,
        editable: true,
      });
    } else {
      shape = new this.mapsLibrary.Polygon({
        paths: points,
        geodesic: true,
        fillColor: color,
        strokeColor: d3color(color)?.darker(0.15).toString(),
        editable: true,
      });
    }
    shape.setMap(this.map);
    shape.getPath().addListener('set_at', () => {
      this.emit('shapeUpdate', shape.getPath());
    });
    shape.getPath().addListener('insert_at', () => {
      this.emit('shapeUpdate', shape.getPath());
    });
    shape.getPath().addListener('remove_at', () => {
      this.emit('shapeUpdate', shape.getPath());
    });
    const s = new Shape({
      type,
      points: JSON.stringify(points),
      layer: '',
    });
    this.shapes[s.Id] = shape;
    return s.Id;
  }

  /**
   * Removes a shape from the map.
   * @param id - The ID of the shape.
   */
  public removeShape(id: string) {
    this.shapes[id].setMap(null);
    delete this.shapes[id];
  }

  /**
   * Initializes the map.
   * @param container - The element to initialize within.
   */
  public async initialize(container: HTMLElement) {
    return new Promise<void>((resolve) => {
      new Loader({
        apiKey: import.meta.env.VITE_GOOGLE_MAPS_KEY,
        version: 'weekly',
      })
        .load()
        .then(async () => {
          this.mapsLibrary = (await google.maps.importLibrary('maps')) as google.maps.MapsLibrary;
          this.visualizationLibrary = (await google.maps.importLibrary(
            'visualization',
          )) as google.maps.VisualizationLibrary;
          this.markerLibrary = (await google.maps.importLibrary(
            'marker',
          )) as google.maps.MarkerLibrary;

          this.map = new this.mapsLibrary.Map(container, {
            zoom: 6,
            mapId: 'DEMO_MAP_ID',
          });

          navigator.geolocation.getCurrentPosition((position: GeolocationPosition) => {
            this.map.setCenter({
              lat: position.coords.latitude,
              lng: position.coords.longitude,
            });
          });

          this.map.addListener('dblclick', (e: google.maps.MapMouseEvent) => {
            const location = e.latLng?.toJSON() as google.maps.LatLngLiteral;
            this.emit('dblclick', location);
          });

          this.map.addListener('click', (e: google.maps.MapMouseEvent) => {
            const location = e.latLng?.toJSON() as google.maps.LatLngLiteral;
            this.emit('click', location);
          });

          resolve();
        });
    });
  }

  /**
   * Hides all markers.
   */
  public hideAllMarkers() {
    Object.values(this.markers).forEach((marker) => {
      marker.el.map = null;
    });
  }

  /**
   * Deletes all markers.
   */
  public clearMarkers() {
    Object.values(this.markers).forEach((marker) => {
      marker.el.map = null;
    });
    this.markers = {};
  }

  /**
   * Hides the heatmap.
   */
  public hideHeatmap() {
    this.heatmap.setMap(null);
  }

  /**
   * Shows all markers.
   */
  public showAllMarkers() {
    Object.values(this.markers).forEach((marker) => {
      marker.el.map = this.map;
    });
  }

  /**
   * Shows the heatmap.
   */
  public showHeatmap() {
    this.heatmap.setMap(this.map);
  }
}
