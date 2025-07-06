import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [selectedBuilding, setSelectedBuilding] = useState<string>("");
  const [selectedRoom, setSelectedRoom] = useState<string>("");
  const [bssids, setBssids] = useState<string[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");

  const buildings = ["A", "B", "C", "D"];
  const rooms = ["101", "102", "201", "202"];

  const getBssids = async () => {
    if (!selectedBuilding || !selectedRoom) {
      setError("建物と教室を選択してください");
      return;
    }

    setLoading(true);
    setError("");
    setBssids([]);

    try {
      const result = await invoke<string[]>("get_bssids", {
        building: selectedBuilding,
        room: selectedRoom,
      });
      setBssids(result);
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
            onClick={getBssids}
            disabled={loading || !selectedBuilding || !selectedRoom}
            className="scan-button"
          >
            {loading ? "スキャン中..." : "BSSID を取得"}
          </button>
        </div>

        {error && <div className="error">{error}</div>}

        {bssids.length > 0 && (
          <div className="results-section">
            <h2>取得した BSSID:</h2>
            <div className="bssid-list">
              {bssids.map((bssid, index) => (
                <div key={index} className="bssid-item">
                  {bssid}
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