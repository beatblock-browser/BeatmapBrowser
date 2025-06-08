import { create } from 'zustand';
import { BeatMap } from '@/schema';

interface AppState {
  selectedMap: BeatMap | null;
  setSelectedMap: (map: BeatMap | null) => void;
}

export const useStore = create<AppState>((set) => ({
  selectedMap: null,
  setSelectedMap: (map) => set({ selectedMap: map }),
})); 