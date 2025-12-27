// Terrain persistence using IndexedDB
// Stores 10,000 x 10,000 terrain grid (~95MB) for fast loading

class TerrainStorage {
  constructor() {
    this.dbName = 'libreterra-db';
    this.storeName = 'terrain';
    this.version = 1;
    this.db = null;
  }

  // Initialize IndexedDB
  async init() {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.version);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = event.target.result;
        if (!db.objectStoreNames.contains(this.storeName)) {
          db.createObjectStore(this.storeName);
        }
      };
    });
  }

  // Save terrain data
  async saveTerrain(terrainGrid, seed) {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);

      const data = {
        width: terrainGrid.width,
        height: terrainGrid.height,
        data: terrainGrid.data,  // Uint8Array
        seed: seed,
        timestamp: Date.now()
      };

      const request = store.put(data, 'current-terrain');
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  // Load terrain data
  async loadTerrain() {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction([this.storeName], 'readonly');
      const store = transaction.objectStore(this.storeName);
      const request = store.get('current-terrain');

      request.onsuccess = () => {
        if (request.result) {
          resolve(request.result);
        } else {
          resolve(null);  // No cached terrain
        }
      };
      request.onerror = () => reject(request.error);
    });
  }

  // Clear terrain data
  async clearTerrain() {
    return new Promise((resolve, reject) => {
      const transaction = this.db.transaction([this.storeName], 'readwrite');
      const store = transaction.objectStore(this.storeName);
      const request = store.delete('current-terrain');

      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });
  }

  // Check if terrain exists
  async hasTerrain() {
    const data = await this.loadTerrain();
    return data !== null;
  }
}

console.log('✓ storage.js loaded successfully');
