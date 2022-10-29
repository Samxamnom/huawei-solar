import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm") version "1.7.0"
    application
}

group = "me.samxamnam"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
    maven {
        url = uri("https://maven.mangoautomation.net/repository/ias-release")
    }
}

dependencies {
    testImplementation(kotlin("test"))
    implementation("com.infiniteautomation:modbus4j:3.0.3")

    implementation("com.google.code.gson:gson:2.10")
//    implementation("org.slf4j:slf4j-api:2.0.3")
//    implementation("org.slf4j:slf4j-simple:2.0.3")
}

tasks.test {
    useJUnitPlatform()
}

tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = "11"
}

application {
    mainClass.set("MainKt")
}