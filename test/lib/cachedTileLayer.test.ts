// AI-generated (Claude)
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock Leaflet before importing the module under test
vi.mock('leaflet', () => ({
  default: {
    TileLayer: class {
      getTileUrl(coords: { z: number; x: number; y: number }) {
        return `https://tile.test/${coords.z}/${coords.x}/${coords.y}.png`;
      }
    },
  },
}));

import { CachedTileLayer } from '$lib/map/cachedTileLayer.ts';

// Shared mock state for IDB
const mockPutReq: any = {};
const mockGetReq: any = {};
const mockStore = {
  get: vi.fn(() => mockGetReq),
  put: vi.fn(() => mockPutReq),
};
const mockDb: any = {
  transaction: vi.fn(() => ({ objectStore: vi.fn(() => mockStore) })),
  createObjectStore: vi.fn(),
};
const mockOpenReq: any = { result: mockDb };

vi.stubGlobal('indexedDB', {
  open: vi.fn(() => {
    queueMicrotask(() => mockOpenReq.onsuccess?.());
    return mockOpenReq;
  }),
});

vi.stubGlobal('URL', {
  createObjectURL: vi.fn(() => 'blob:fake'),
  revokeObjectURL: vi.fn(),
});

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
  // Reset IDB mock callbacks
  delete mockOpenReq.onsuccess;
  delete mockGetReq.onsuccess;
  delete mockGetReq.onerror;
  delete mockPutReq.onsuccess;
  delete mockPutReq.onerror;
});

const COORDS = { z: 5, x: 10, y: 20, equals: () => false, scaleBy: () => ({} as any), unscaleBy: () => ({} as any) } as any;

describe('CachedTileLayer', () => {
  it('cache hit: returns blob URL without calling fetch', () => {
    return new Promise<void>((resolve, reject) => {
      const cachedBuf = new ArrayBuffer(4);

      // IDB open resolves on next microtask (handled by stubGlobal above)
      vi.mocked(indexedDB.open).mockImplementation(() => {
        queueMicrotask(() => mockOpenReq.onsuccess?.());
        return mockOpenReq;
      });
      mockStore.get.mockImplementation(() => {
        const req: any = { result: cachedBuf };
        queueMicrotask(() => req.onsuccess?.());
        return req;
      });

      const layer = new CachedTileLayer('https://tile/{z}/{x}/{y}.png');
      layer.createTile(COORDS, (err, img) => {
        try {
          expect(err).toBeUndefined();
          expect(mockFetch).not.toHaveBeenCalled();
          expect((URL.createObjectURL as ReturnType<typeof vi.fn>)).toHaveBeenCalled();
          resolve();
        } catch (e) { reject(e); }
      });
    });
  });

  it('cache miss: fetches, stores in IDB, returns blob URL', () => {
    return new Promise<void>((resolve, reject) => {
      const fetchedBuf = new ArrayBuffer(8);

      vi.mocked(indexedDB.open).mockImplementation(() => {
        queueMicrotask(() => mockOpenReq.onsuccess?.());
        return mockOpenReq;
      });
      // get returns null (cache miss)
      mockStore.get.mockImplementation(() => {
        const req: any = { result: null };
        queueMicrotask(() => req.onsuccess?.());
        return req;
      });
      // put succeeds
      mockStore.put.mockImplementation(() => {
        const req: any = {};
        queueMicrotask(() => req.onsuccess?.());
        return req;
      });
      mockFetch.mockResolvedValue({ arrayBuffer: () => Promise.resolve(fetchedBuf) });

      const layer = new CachedTileLayer('https://tile/{z}/{x}/{y}.png');
      layer.createTile(COORDS, (err, img) => {
        try {
          expect(err).toBeUndefined();
          expect(mockFetch).toHaveBeenCalledOnce();
          expect(mockStore.put).toHaveBeenCalledWith(fetchedBuf, expect.any(String));
          expect((URL.createObjectURL as ReturnType<typeof vi.fn>)).toHaveBeenCalled();
          resolve();
        } catch (e) { reject(e); }
      });
    });
  });

  it('network error: calls done with the error', () => {
    return new Promise<void>((resolve, reject) => {
      vi.mocked(indexedDB.open).mockImplementation(() => {
        queueMicrotask(() => mockOpenReq.onsuccess?.());
        return mockOpenReq;
      });
      mockStore.get.mockImplementation(() => {
        const req: any = { result: null };
        queueMicrotask(() => req.onsuccess?.());
        return req;
      });
      const networkError = new Error('offline');
      mockFetch.mockRejectedValue(networkError);

      const layer = new CachedTileLayer('https://tile/{z}/{x}/{y}.png');
      layer.createTile(COORDS, (err) => {
        try {
          expect(err).toBe(networkError);
          resolve();
        } catch (e) { reject(e); }
      });
    });
  });
});
