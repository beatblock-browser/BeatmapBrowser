import { create } from 'zustand';
import { BeatMap } from '@/schema';

interface AppState {
  selectedMap: BeatMap | null;
  setSelectedMap: (map: BeatMap | null) => void;
  upvoteMap: (mapId: string) => Promise<void>;
  unvoteMap: (mapId: string) => Promise<void>;
}

export const useStore = create<AppState>((set) => ({
  selectedMap: null,
  setSelectedMap: (map) => set({ selectedMap: map }),
  upvoteMap: async (mapId) => {
    try {
      const response = await fetch('/api/upvote', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ mapId }),
      });
      if (!response.ok) throw new Error('Failed to upvote');
      
      set((state) => {
        if (!state.selectedMap) return state;
        return {
          selectedMap: {
            ...state.selectedMap,
            upvotes: state.selectedMap.upvotes + 1,
          },
        };
      });
    } catch (error) {
      console.error('Error upvoting:', error);
    }
  },
  unvoteMap: async (mapId) => {
    try {
      const response = await fetch('/api/unvote', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ mapId }),
      });
      if (!response.ok) throw new Error('Failed to unvote');
      
      set((state) => {
        if (!state.selectedMap) return state;
        return {
          selectedMap: {
            ...state.selectedMap,
            upvotes: state.selectedMap.upvotes - 1,
          },
        };
      });
    } catch (error) {
      console.error('Error unvoting:', error);
    }
  },
})); 