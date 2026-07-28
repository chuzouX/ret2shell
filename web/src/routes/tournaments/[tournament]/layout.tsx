import { getTournament } from "@api/tournament";
import { useParams } from "@solidjs/router";
import { Title } from "@storage/header";
import { t } from "@storage/theme";
import { createResource, type JSX } from "solid-js";

export default function (props: { children?: JSX.Element }) {
  const params = useParams();
  const id = () => Number(params.tournament);
  const [tournament] = createResource(id, getTournament);
  return (
    <>
      <Title domain={tournament()?.name || t("tournament.title")} route={`/tournaments/${id()}`} />
      {props.children}
    </>
  );
}
