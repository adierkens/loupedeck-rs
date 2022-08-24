const { LoupedeckDevice } = require("loupedeck");

// Detects and opens first connected device
const device = new LoupedeckDevice({
  path: "COM3",
});

const main = async () => {
  console.log(await device.getInfo());
  // device.vibrate();

  device.drawKey(2, (ctx) => {
    ctx.fillStyle = "red";
    ctx.fillRect(0, 0, 45, 45);
  });
};

device.on("connect", () => {
  main();
});
