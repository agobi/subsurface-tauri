// AI-generated (Claude)
import L from 'leaflet';

let db: IDBDatabase | null = null;

async function openDb(): Promise<IDBDatabase> {
  if (db) return db;
  return new Promise<IDBDatabase>((resolve, reject) => {
    const req = indexedDB.open('subsurface-tiles', 1);
    req.onupgradeneeded = () => req.result.createObjectStore('tiles');
    req.onsuccess = () => { db = req.result; resolve(req.result); };
    req.onerror = () => reject(req.error);
  });
}

async function getCached(url: string): Promise<ArrayBuffer | null> {
  const database = await openDb();
  return new Promise<ArrayBuffer | null>((resolve, reject) => {
    const tx = database.transaction('tiles', 'readonly');
    const req = tx.objectStore('tiles').get(url);
    req.onsuccess = () => resolve((req.result as ArrayBuffer) ?? null);
    req.onerror = () => reject(req.error);
  });
}

async function putCached(url: string, buf: ArrayBuffer): Promise<void> {
  const database = await openDb();
  return new Promise<void>((resolve, reject) => {
    const tx = database.transaction('tiles', 'readwrite');
    const req = tx.objectStore('tiles').put(buf, url);
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
  });
}

export class CachedTileLayer extends L.TileLayer {
  override createTile(coords: L.Coords, done: L.DoneCallback): HTMLElement {
    const img = document.createElement('img');
    img.alt = '';
    const url = this.getTileUrl(coords);

    (async () => {
      try {
        const cached = await getCached(url);
        if (cached !== null) {
          img.src = URL.createObjectURL(new Blob([cached]));
          done(undefined, img);
          return;
        }
        const resp = await fetch(url);
        const buf = await resp.arrayBuffer();
        await putCached(url, buf);
        img.src = URL.createObjectURL(new Blob([buf]));
        done(undefined, img);
      } catch (err) {
        done(err as Error, img);
      }
    })();

    return img;
  }
}

export function cachedTileLayer(urlTemplate: string, options?: L.TileLayerOptions): L.TileLayer {
  return new CachedTileLayer(urlTemplate, options);
}
