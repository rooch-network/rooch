import "../styles.css";
import "nextra-theme-docs/style.css";
import { GoogleAnalytics } from "nextjs-google-analytics";
import { useEffect } from "react";

export default function Nextra({ Component, pageProps }) {
  useEffect(() => {
    import("preline");
  }, []);
  return (
    <>
      <GoogleAnalytics trackPageViews />
      <Component {...pageProps} />
    </>
  );
}
