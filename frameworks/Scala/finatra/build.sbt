lazy val finatraVersion = "22.7.0"

name := "techempower-benchmarks-finatra"
organization := "com.twitter"
version := finatraVersion

scalaVersion := "2.12.20"

resolvers ++= Seq(
  Resolver.sonatypeRepo("releases")
)

assemblyJarName in assembly := "finatra-benchmark.jar"
assemblyMergeStrategy in assembly := {
  case "BUILD" => MergeStrategy.discard
  case PathList("META-INF", "io.netty.versions.properties") => MergeStrategy.discard
  case PathList("META-INF", "MANIFEST.MF") => MergeStrategy.discard
  case other => MergeStrategy.first}

libraryDependencies ++= Seq(
  ("com.twitter" %% "finatra-http-server" % finatraVersion).
    exclude("com.sun.activation", "javax.activation"),
  "org.slf4j" % "slf4j-nop" % "1.7.30",
  "com.fasterxml.jackson.module" %% "jackson-module-scala" % "2.13.1",
)

excludeDependencies ++= Seq(
  // commons-logging is replaced by jcl-over-slf4j
  ExclusionRule("org.slf4j", "slf4j-simple"),
  ExclusionRule("commons-logging", "commons-logging")
)
