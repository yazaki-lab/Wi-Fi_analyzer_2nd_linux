import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface WifiNetwork {
  ssid: string;
  bssid: string;
  signal: string;
}

function App() {
  const [selectedBuilding, setSelectedBuilding] = useState<string>("");
  const [selectedRoom, setSelectedRoom] = useState<string>("");
  const [networks, setNetworks] = useState<WifiNetwork[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");

  const buildings = ["A", "B", "C", "D"];
  const rooms = ["101", "102", "201", "202"];

  const getWifiNetworks = async () => {
    if (!selectedBuilding || !selectedRoom) {
      setError("建物と教室を選択してください");
      return;
    }

    setLoading(true);
    setError("");
    setNetworks([]);

    try {
      const result = await invoke<WifiNetwork[]>("get_bssids", {
        building: selectedBuilding,
        room: selectedRoom,
      });
      setNetworks(result);
    } catch (err) {
      setError(`エラーが発生しました: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container">
      <header>
        <h1>wi-fi.viewer</h1>
      </header>

      <main>
        <div className="form-section">
          <div className="form-group">
            <label htmlFor="building">建物を選択:</label>
            <select
              id="building"
              value={selectedBuilding}
              onChange={(e) => setSelectedBuilding(e.target.value)}
            >
              <option value="">-- 建物を選択 --</option>
              {buildings.map((building) => (
                <option key={building} value={building}>
                  {building}棟
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="room">教室を選択:</label>
            <select
              id="room"
              value={selectedRoom}
              onChange={(e) => setSelectedRoom(e.target.value)}
            >
              <option value="">-- 教室を選択 --</option>
              {rooms.map((room) => (
                <option key={room} value={room}>
                  {room}
                </option>
              ))}
            </select>
          </div>

          <button
            onClick={getWifiNetworks}
            disabled={loading || !selectedBuilding || !selectedRoom}
            className="scan-button"
          >
            {loading ? "スキャン中..." : "Wi-Fi ネットワークを取得"}
          </button>
        </div>

        {error && <div className="error">{error}</div>}

        {networks.length > 0 && (
          <div className="results-section">
            <h2>検出されたWi-Fiネットワーク:</h2>
            <div className="network-list">
              {networks.map((network, index) => (
                <div key={index} className="network-item">
                  <div className="network-info">
                    <div className="ssid">
                      <strong>SSID:</strong> {network.ssid}
                    </div>
                    <div className="bssid">
                      <strong>BSSID:</strong> {network.bssid}
                    </div>
                    <div className="signal">
                      <strong>信号強度:</strong> {network.signal} dBm
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;