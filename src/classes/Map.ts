import type { ShapeType } from './Shape';
import { Loader } from '@googlemaps/js-api-loader';
import { color as d3color } from 'd3-color';
import { EventEmitter } from 'ee-ts';

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
  airport: 'mdi-airplane',
  ambulance: 'mdi-ambulance',
  'amusement-park': 'mdi-ferris-wheel',
  aquarium: 'mdi-fishbowl',
  arcade: 'mdi-controller-classic',
  art: 'mdi-palette',
  bank: 'mdi-bank',
  bar: 'mdi-glass-cocktail',
  barber: 'mdi-content-cut',
  baseball: 'mdi-baseball',
  basketball: 'mdi-basketball',
  bathroom: 'mdi-toilet',
  beach: 'mdi-beach',
  bike: 'mdi-bike',
  bird: 'mdi-bird',
  boat: 'mdi-sail-boat',
  bowling: 'mdi-bowling',
  bridge: 'mdi-bridge',
  burger: 'mdi-hamburger',
  bus: 'mdi-bus',
  cafe: 'mdi-coffee',
  camping: 'mdi-campfire',
  car: 'mdi-car',
  'car-dealership': 'mdi-car-key',
  cards: 'mdi-cards',
  castle: 'mdi-castle',
  cat: 'mdi-cat',
  cemetery: 'mdi-grave-stone',
  chess: 'mdi-chess-pawn',
  church: 'mdi-cross',
  dentist: 'mdi-tooth',
  doctor: 'mdi-medical-bag',
  factory: 'mdi-factory',
  farm: 'mdi-silo',
  fire: 'mdi-fire',
  fireworks: 'mdi-firework',
  'fire-tower': 'mdi-tower-fire',
  frisbee: 'mdi-disc',
  garden: 'mdi-flower',
  gas: 'mdi-gas-station',
  glasses: 'mdi-glasses',
  golf: 'mdi-golf',
  grocery: 'mdi-food-apple',
  gun: 'mdi-pistol',
  gym: 'mdi-dumbbell',
  hiking: 'mdi-hiking',
  home: 'mdi-home',
  horse: 'mdi-horse',
  hospital: 'mdi-hospital',
  hotel: 'mdi-bed',
  hunting: 'mdi-bow-arrow',
  icecream: 'mdi-ice-cream',
  jellyfish: 'mdi-jellyfish',
  library: 'mdi-library',
  lighthouse: 'mdi-lighthouse',
  mail: 'mdi-mailbox',
  'martial-arts': 'mdi-karate',
  monument: 'mdi-chess-rook',
  mountain: 'mdi-terrain',
  'movie-theater': 'mdi-filmstrip',
  museum: 'mdi-image-filter-frames',
  nuclear: 'mdi-atom',
  office: 'mdi-office-building',
  park: 'mdi-tree',
  parking: 'mdi-parking',
  pharmacy: 'mdi-pill',
  pizza: 'mdi-pizza',
  playground: 'mdi-slide',
  photography: 'mdi-camera',
  pool: 'mdi-swim',
  rabbit: 'mdi-rabbit',
  'radio-tower': 'mdi-radio-tower',
  realtor: 'mdi-sign-real-estate',
  restaurant: 'mdi-silverware-fork-knife',
  rowing: 'mdi-rowing',
  school: 'mdi-school',
  ship: 'mdi-ship-wheel',
  skate: 'mdi-skate',
  smoking: 'mdi-smoking',
  soccer: 'mdi-soccer',
  stadium: 'mdi-stadium',
  store: 'mdi-store',
  subway: 'mdi-subway-variant',
  technology: 'mdi-laptop',
  tennis: 'mdi-tennis-ball',
  theater: 'mdi-theater',
  train: 'mdi-train',
  waterfall: 'mdi-waterfall',
  work: 'mdi-briefcase',
  zoo: 'mdi-elephant',
};

export type PlaceType = keyof typeof icons;

/**
 * Helper method to get lat,lng as a string.
 * @param param0 - The location.
 * @returns The location string.
 */
export function locToString(location?: { lat: number; lng: number }) {
  return location ? `${location.lat.toString()},${location.lng.toString()}` : '';
}

/**
 * Helper method to get lat,lng from a string.
 * @param str - The string.
 * @returns The location object.
 */
export function stringToLoc(str: string) {
  const split = str.split(',').map(Number);
  return {
    lat: split[0] ?? 0,
    lng: split[1] ?? 0,
  };
}

/**
 * Provides functions for working with google maps.
 */
export class Map extends EventEmitter<{
  markerClicked: (place: string) => void;
  markerCreated: (pos: Position) => void;
  click: (pos: Position) => void;
  dblclick: (pos: Position) => void;
  shapeUpdate: (newPath: google.maps.MVCArray<google.maps.LatLng>) => void;
}> {
  static BlankMap = 'c1707ca92a2f3bcc';

  static DefaultMap = '92b62a7797a48aa';

  private container!: HTMLElement;

  private map!: google.maps.Map;

  private mapsLibrary!: google.maps.MapsLibrary;

  private markers: Record<string, Marker> = {};

  private markerLibrary!: google.maps.MarkerLibrary;

  private shapes: Record<string, google.maps.MVCObject> = {};

  /**
   * Places a marker on the map.
   * @param pos - The position to place the marker at.
   * @param icon - An icon to use for the marker.
   * @param color - The color of the marker.
   * @param title - The title of the marker.
   * @param id - The ID of the associated Place, if any.
   */
  public createMarker(
    pos: string,
    id: string,
    icon?: keyof typeof icons,
    color?: string,
    title?: string,
    count?: number,
  ) {
    if (!this.markers[id]) {
      const position = stringToLoc(pos);
      const marker: google.maps.marker.AdvancedMarkerElementOptions = {
        map: this.map,
        position,
        title,
      };
      if (typeof icon === 'string' && typeof color === 'string') {
        const i = document.createElement('div');
        i.innerHTML = `<i class="mdi ${icons[icon]}"></i>`;
        const markerEl = document.createElement('div');
        markerEl.append(
          new this.markerLibrary.PinElement({
            glyph: i,
            background: color,
            borderColor: d3color(color)?.darker(0.15).toString(),
          }).element,
        );
        if (typeof count === 'number' && count > 0) {
          const countEl = document.createElement('div');
          countEl.textContent = count.toString();
          countEl.style.backgroundColor = 'red';
          if (count >= 1000) {
            countEl.style.width = '25px';
          } else if (count >= 100) {
            countEl.style.width = '20px';
          } else if (count >= 10) {
            countEl.style.width = '14px';
          } else {
            countEl.style.width = '12px';
          }
          countEl.style.height = '12px';
          countEl.style.borderRadius = '12px';
          countEl.style.color = 'white';
          countEl.style.position = 'fixed';
          countEl.style.top = '-4px';
          countEl.style.right = '-2px';
          countEl.style.textAlign = 'center';
          markerEl.append(countEl);
        }
        marker.content = markerEl;
      }
      this.markers[id] = {
        el: new this.markerLibrary.AdvancedMarkerElement(marker),
        position,
        count: 1, // TODO
      };
      google.maps.event.addListener(this.markers[id].el, 'click', () => {
        this.emit('markerClicked', id);
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
   * @param color - The shape color.
   * @param id - The Shape id.
   * @param editable - If the shape should be editable.
   */
  public createShape(
    type: ShapeType,
    points: Position[],
    color: string,
    id: string,
    editable = false,
  ) {
    const shape =
      type === 'line'
        ? new this.mapsLibrary.Polyline({
            path: points,
            geodesic: true,
            strokeColor: color,
            strokeWeight: 2,
            editable,
          })
        : new this.mapsLibrary.Polygon({
            paths: points,
            geodesic: true,
            fillColor: color,
            strokeColor: d3color(color)?.darker(0.15).toString(),
            editable,
          });
    shape.setMap(this.map);
    shape.addListener('click', () => {
      console.log(id);
      // TODO: expand this functionality
    });
    if (editable) {
      shape.getPath().addListener('set_at', () => {
        this.emit('shapeUpdate', shape.getPath());
      });
      shape.getPath().addListener('insert_at', () => {
        this.emit('shapeUpdate', shape.getPath());
      });
      shape.getPath().addListener('remove_at', () => {
        this.emit('shapeUpdate', shape.getPath());
      });
    }
    this.shapes[id] = shape;
    return shape;
  }

  /**
   * Removes a shape from the map.
   * @param id - The ID of the shape.
   */
  public removeShape(id: string) {
    (this.shapes[id] as google.maps.Polyline).setMap(null);
    delete this.shapes[id];
  }

  /**
   * Deletes a marker.
   * @param id - The ID of the marker to delete.
   */
  public removeMarker(id: string) {
    if (this.markers[id]) {
      this.markers[id].el.map = null;
      delete this.markers[id];
    }
  }

  /**
   * Initializes the map.
   * @param container - The element to initialize within.
   * @param style - Map ID style to initialize with.
   */
  public async initialize(container: HTMLElement, style = Map.DefaultMap) {
    this.container = container;
    const loader = new Loader({
      apiKey: import.meta.env.VITE_GOOGLE_MAPS_KEY as string,
      version: 'weekly',
    });
    this.mapsLibrary = await loader.importLibrary('maps');
    this.markerLibrary = await loader.importLibrary('marker');

    this.map = new this.mapsLibrary.Map(container, {
      zoom: 6,
      mapId: style,
    });
    this.map.setCenter({ lat: 0, lng: 0 });
    navigator.geolocation.getCurrentPosition(position => {
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
  }

  /**
   * Hides all markers.
   */
  public hideAllMarkers() {
    for (const marker of Object.values(this.markers)) {
      marker.el.map = null;
    }
  }

  /**
   * Deletes all markers.
   */
  public clearMarkers() {
    for (const marker of Object.values(this.markers)) {
      marker.el.map = null;
    }
    this.markers = {};
  }

  /**
   * Shows all markers.
   */
  public showAllMarkers() {
    for (const marker of Object.values(this.markers)) {
      marker.el.map = this.map;
    }
  }

  /**
   * Changes the map style.
   * @param style - The map ID corresponding to the desired style.
   */
  public async setStyle(style: string) {
    await this.initialize(this.container, style);
    for (const marker of Object.values(this.markers)) {
      marker.el.map = this.map;
    }
    for (const shape of Object.values(this.shapes)) {
      (shape as google.maps.Polygon).setMap(this.map);
    }
  }

  public setCenter(lat: number, lng: number) {
    this.map.setCenter({
      lat,
      lng,
    });
  }
}
