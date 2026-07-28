import type { JSX } from "solid-js";
import Cover from "./_blocks/cover";

export default function (props: { children?: JSX.Element }) {
  return (
    <>
      {props.children}
      <Cover />
    </>
  );
}
