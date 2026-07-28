import { Title } from "@storage/header";
import { t } from "@storage/theme";
import ErrorSection from "./error";

export default function () {
  return (
    <>
      <Title page={t("general.network.status.404.title")} route="/error/404" />
      <ErrorSection status={404} />
    </>
  );
}
