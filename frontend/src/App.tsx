import { useState } from "react"

type Device = {
  user: String,
  os: String,
  id: String,
}


function App() {

  const [device, setDevice] = useState<Device[]>([]);


  const fetchUsers = async () => {
    const res = await fetch("http://127.0.0.1:8081/api/scan-devices");
    const json: Device[] = await res.json();
    setDevice(json);
  };

  return (
    <>
      <button onClick={fetchUsers}>Scan Devices</button>
      <hr />
      <h2>Online Device List</h2>
      <div>
        {device.map((v, i) => (
          <div key={i}>
            <p>User: {v.user}</p>
            <p>Os: {v.os}</p>
            <p>Id: {v.id}</p>
          </div>
        ))}
      </div>
      <hr />
      <h2>All Device List</h2>
    </>
  )
}

export default App
