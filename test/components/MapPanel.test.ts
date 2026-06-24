// AI-generated (Claude)
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { flushSync } from 'svelte';
import MapPanel from '$lib/components/MapPanel.svelte';

const mockRemove = vi.fn();
const mockFlyTo = vi.fn();
const mockSetView = vi.fn();
const mockMarkerSetLatLng = vi.fn();
const mockMarkerBindPopup = vi.fn();
const mockTileLayerAddTo = vi.fn();
const mockMarkerAddTo = vi.fn();

const mockMapInstance = {
  setView: mockSetView,
  flyTo: mockFlyTo,
  remove: mockRemove,
};

vi.mock('leaflet', () => ({
  default: {
    map: vi.fn(() => mockMapInstance),
    divIcon: vi.fn(() => ({})),
    marker: vi.fn(() => ({
      addTo: mockMarkerAddTo,
      setLatLng: mockMarkerSetLatLng,
      bindPopup: mockMarkerBindPopup,
    })),
  },
}));

vi.mock('$lib/map/cachedTileLayer.ts', () => ({
  cachedTileLayer: vi.fn(() => ({ addTo: mockTileLayerAddTo })),
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockSetView.mockReturnValue(mockMapInstance);
  mockFlyTo.mockReturnValue(mockMapInstance);
  mockMarkerAddTo.mockReturnValue({
    setLatLng: mockMarkerSetLatLng,
    bindPopup: mockMarkerBindPopup,
  });
});

describe('MapPanel', () => {
  it('renders the map container and caption when gps is present', () => {
    const { container } = render(MapPanel, {
      props: { siteName: 'Fenyes Forras', gps: { lat: 47.66, lon: 18.3 } },
    });
    expect(container.querySelector('.map-inner')).toBeInTheDocument();
    expect(screen.getByText(/Fenyes Forras/)).toBeInTheDocument();
    expect(screen.getByText(/47.660/)).toBeInTheDocument();
  });

  it('shows "No GPS for this site" when site has no gps', () => {
    render(MapPanel, { props: { siteName: 'Fenyes Forras', gps: undefined } });
    expect(screen.getByText(/no gps for this site/i)).toBeInTheDocument();
  });

  it('shows "No site for this dive" when no site', () => {
    render(MapPanel, { props: { siteName: undefined, gps: undefined } });
    expect(screen.getByText(/no site for this dive/i)).toBeInTheDocument();
  });

  it('calls map.remove() on destroy', () => {
    const { unmount } = render(MapPanel, {
      props: { siteName: 'Fenyes Forras', gps: { lat: 47.66, lon: 18.3 } },
    });
    flushSync();
    unmount();
    expect(mockRemove).toHaveBeenCalled();
  });
});
