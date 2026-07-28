import { Title } from "@storage/header";
import { t } from "@storage/theme";
import ErrorSection from "./error";

export default function () {
  return (
    <>
      <Title page={t("general.network.unknown.title")} route="/error/unknown" />
      <ErrorSection />
    </>
  );
}
