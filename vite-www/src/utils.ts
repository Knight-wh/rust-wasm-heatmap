function heatToColor(
  heat: number,
  gradient: { r: number; g: number; b: number }[],
  maxHeat: number
) {
  let r = 0,
    g = 0,
    b = 0,
    a = 0;
  const gradientSteps = gradient.length;
  const stepLens = maxHeat / gradientSteps;

  const heatStepF = Math.floor(heat / stepLens);
  const stepPos = heat / stepLens - heatStepF;

  if (heatStepF >= gradientSteps) {
    r = gradient[gradientSteps - 1].r;
    g = gradient[gradientSteps - 1].g;
    b = gradient[gradientSteps - 1].b;
    a = 255;
  } else {
    if (heatStepF === 0) {
      r = gradient[0].r;
      g = gradient[0].g;
      b = gradient[0].b;
      a = Math.round(255 * stepPos); // 我以为会使用插值
    } else {
      const gradPosInv = 1 - stepPos;
      r = Math.round(
        gradient[heatStepF - 1].r * gradPosInv + gradient[heatStepF].r * stepPos
      );
      g = Math.round(
        gradient[heatStepF - 1].g * gradPosInv + gradient[heatStepF].g * stepPos
      );
      b = Math.round(
        gradient[heatStepF - 1].b * gradPosInv + gradient[heatStepF].b * stepPos
      );
      a = 255;
    }
  }

  return [r, g, b, a];
}

function parseGradient(
  gradient: string[] | number[][]
): { r: number; g: number; b: number }[] {
  return gradient.map((color) => {
    if (color.toString().match(/^#?[0-9a-f]{3}$/i)) {
      color = color.toString().replace(/^#?(.)(.)(.)$/, "$1$1$2$2$3$3");
    }
    if (typeof color === "string") {
      if (color.match(/^#?[0-9a-f]{6}$/i)) {
        // @ts-ignore
        color = color
          .match(/^#?(..)(..)(..)$/)
          .slice(1)
          .map((n) => parseInt(n, 16));
      } else {
        throw Error(`Invalid color format (${color}).`);
      }
    } else if (color instanceof Array) {
      if (
        !(
          color.length &&
          isUint8(color[0]) &&
          isUint8(color[1]) &&
          isUint8(color[2])
        )
      ) {
        throw Error(`Invalid color format (${JSON.stringify(color)}).`);
      }
    } else {
      throw Error(`Invalid color object (${JSON.stringify(color)}).`);
    }
    return { r: color[0], g: color[1], b: color[2] };
  });
}

function isUint8(num: number) {
  return typeof num == "number" && 0 <= num && num >= 255;
}

export { heatToColor, parseGradient, isUint8 };
