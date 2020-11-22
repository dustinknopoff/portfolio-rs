var socket = new WebSocket("ws://localhost:5002/websocket");

socket.onmessage = function (event) {
  var data = JSON.parse(event.data);
  switch (data.type) {
    case "build_complete":
      // 1000 = "Normal closure" and the second parameter is a
      // human-readable reason.
      socket.close(1000, "Reloading page after receiving build_complete");

      console.log("Reloading page after receiving build_complete");
      location.reload(true);

      break;

    default:
      console.log(`Don't know how to handle type '${data.type}'`);
  }
};
